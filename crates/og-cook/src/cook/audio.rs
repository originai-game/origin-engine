//! WAV → QOA sound blobs.

use super::CookError;
use crate::format::{self, *};
use crate::qoa;
use og_blob::BlobWriter;

/// Cooks a WAV file (8/16/24/32-bit integer or 32-bit float PCM, 1 to 8
/// channels) into a sound blob.
pub fn cook(bytes: &[u8], stamp: u64) -> Result<(Vec<u8>, String), CookError> {
    let reader = hound::WavReader::new(std::io::Cursor::new(bytes))
        .map_err(|e| CookError(format!("WAV: {e}")))?;
    let spec = reader.spec();
    let channels = spec.channels as usize;
    let samples: Vec<i16> = match (spec.sample_format, spec.bits_per_sample) {
        (hound::SampleFormat::Float, 32) => reader
            .into_samples::<f32>()
            .map(|s| s.map(|v| (v.clamp(-1.0, 1.0) * 32767.0).round() as i16))
            .collect::<Result<_, _>>(),
        (hound::SampleFormat::Int, bits @ 1..=16) => reader
            .into_samples::<i32>()
            .map(|s| s.map(|v| (v << (16 - bits)) as i16))
            .collect::<Result<_, _>>(),
        (hound::SampleFormat::Int, bits @ 17..=32) => reader
            .into_samples::<i32>()
            .map(|s| s.map(|v| (v >> (bits - 16)) as i16))
            .collect::<Result<_, _>>(),
        (f, b) => {
            return Err(CookError(format!(
                "WAV: {b}-bit {f:?} samples are not supported"
            )));
        }
    }
    .map_err(|e| CookError(format!("WAV: {e}")))?;
    let qoa = qoa::encode(&samples, channels, spec.sample_rate).map_err(CookError)?;
    let frames = (samples.len() / channels.max(1)) as u32;
    let meta = SoundMeta {
        sample_rate: spec.sample_rate,
        channels: channels as u32,
        frames,
    };
    let note = format!(
        "{} Hz, {channels} channel(s), {:.2} s, {}-bit PCM to QOA",
        spec.sample_rate,
        frames as f64 / spec.sample_rate.max(1) as f64,
        spec.bits_per_sample
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
