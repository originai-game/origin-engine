//! WAV → QOA sound blobs, whole or in chunks.
//!
//! A long sound (`sound { stream true }`, or a QOA file above
//! [`STREAM_BYTES`] unless `stream false`) is cooked in chunks of whole QOA
//! frames, about `chunk` seconds each: the sound's blob holds the QOA
//! header and the first chunk, and every other chunk is a blob of its own
//! (`<id>#chunk<n>`, `og.SoundChunk`) that the runtime reads ahead of the
//! playhead and drops behind it. The file is encoded once and cut between
//! frames: a QOA frame carries its predictor's state, so the chunks decode
//! to exactly the samples of the whole file.

use super::CookError;
use crate::format::{self, *};
use crate::qoa;
use og_blob::BlobWriter;

/// A QOA file above this many bytes is cooked in chunks unless its import
/// settings say `stream false` (1 MiB: 27 s of 48 kHz stereo).
pub const STREAM_BYTES: u64 = 1 << 20;

/// Samples per channel in a full QOA frame.
const QOA_FRAME: u64 = 5120;

/// `sound { … }` import settings of a WAV.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SoundSettings {
    /// Cooked in chunks: `Some(true)` always, `Some(false)` never, `None`
    /// above [`STREAM_BYTES`].
    pub stream: Option<bool>,
    /// Seconds per chunk (rounded up to whole QOA frames), 1..=60.
    pub chunk: f32,
}

impl Default for SoundSettings {
    fn default() -> Self {
        SoundSettings {
            stream: None,
            chunk: 4.0,
        }
    }
}

impl SoundSettings {
    pub fn parse(block: Option<&og_core::ogt::Block>) -> Result<SoundSettings, String> {
        let mut s = SoundSettings::default();
        let Some(b) = block else { return Ok(s) };
        for e in &b.entries {
            match e.key.as_str() {
                "stream" => {
                    s.stream =
                        Some(e.values.first().and_then(|v| v.as_bool()).ok_or_else(|| {
                            format!("line {}: stream takes true or false", e.line)
                        })?)
                }
                "chunk" => {
                    let c = e.f32(0).map_err(|x| format!("line {}: {x}", e.line))?;
                    if !(1.0..=60.0).contains(&c) {
                        return Err(format!("line {}: chunk must be 1..=60 seconds", e.line));
                    }
                    s.chunk = c;
                }
                k => {
                    return Err(format!(
                        "line {}: unknown sound setting '{k}' (stream, chunk)",
                        e.line
                    ));
                }
            }
        }
        Ok(s)
    }

    pub fn describe(&self) -> String {
        format!(
            "stream {} chunk {}",
            self.stream.map_or("auto".to_string(), |b| b.to_string()),
            self.chunk
        )
    }

    /// QOA frames per chunk and the chunk count for a WAV of this shape,
    /// or `None` to cook it whole.
    pub fn chunking(&self, rate: u32, channels: u32, frames: u64) -> Option<(u32, u32)> {
        let qoa_frames = frames.div_ceil(QOA_FRAME).max(1);
        let ch = u64::from(channels.max(1));
        // A full frame: header, the LMS state per channel, 256 slices per channel.
        let bytes = 8 + qoa_frames * (8 + 16 * ch + 8 * 256 * ch);
        let wanted = self.stream.unwrap_or(bytes > STREAM_BYTES);
        if !wanted {
            return None;
        }
        let per = ((f64::from(self.chunk) * f64::from(rate)) / QOA_FRAME as f64).ceil() as u64;
        let per = per.max(1);
        Some((per as u32, qoa_frames.div_ceil(per) as u32))
    }
}

/// The shape of a WAV from its header: rate, channels, samples per channel.
pub fn shape(bytes: &[u8]) -> Result<(u32, u32, u64), String> {
    let r = hound::WavReader::new(std::io::Cursor::new(bytes)).map_err(|e| format!("WAV: {e}"))?;
    let spec = r.spec();
    Ok((
        spec.sample_rate,
        u32::from(spec.channels),
        u64::from(r.duration()),
    ))
}

/// A WAV encoded to QOA, once per cook (its chunks share it).
#[derive(Debug)]
pub struct Encoded {
    pub qoa: Vec<u8>,
    pub meta: SoundMeta,
    pub note: String,
    /// Cooked in chunks: (QOA frames per chunk, the chunks as
    /// `qoa::split` cuts them).
    pub chunks: Option<(u32, Vec<Vec<u8>>)>,
}

/// Encodes a WAV file (8/16/24/32-bit integer or 32-bit float PCM, 1 to 8
/// channels) to QOA, cut into chunks of `per_chunk` QOA frames when given.
pub fn encode(bytes: &[u8], per_chunk: Option<u32>) -> Result<Encoded, String> {
    let reader =
        hound::WavReader::new(std::io::Cursor::new(bytes)).map_err(|e| format!("WAV: {e}"))?;
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
            return Err(format!("WAV: {b}-bit {f:?} samples are not supported"));
        }
    }
    .map_err(|e| format!("WAV: {e}"))?;
    let qoa = qoa::encode(&samples, channels, spec.sample_rate)?;
    let frames = (samples.len() / channels.max(1)) as u32;
    let chunks = match per_chunk {
        Some(per) => Some((per, qoa::split(&qoa, per as usize)?)),
        None => None,
    };
    Ok(Encoded {
        chunks,
        qoa,
        meta: SoundMeta {
            sample_rate: spec.sample_rate,
            channels: channels as u32,
            frames,
            ..SoundMeta::default()
        },
        note: format!(
            "{} Hz, {channels} channel(s), {:.2} s, {}-bit PCM to QOA",
            spec.sample_rate,
            frames as f64 / spec.sample_rate.max(1) as f64,
            spec.bits_per_sample
        ),
    })
}

/// The sound's blob: the whole QOA file, or with `chunking` (QOA frames
/// per chunk, chunks) its header and first chunk.
pub fn cook(
    e: &Encoded,
    chunking: Option<(u32, u32)>,
    stamp: u64,
) -> Result<(Vec<u8>, String), CookError> {
    let mut meta = e.meta;
    let mut note = e.note.clone();
    let qoa = match (chunking, &e.chunks) {
        (None, _) => e.qoa.clone(),
        (Some(_), None) => return Err(CookError("encoded whole, planned in chunks".into())),
        (Some((per, count)), Some((_, parts))) => {
            if parts.len() != count as usize {
                return Err(CookError(format!(
                    "{} chunks where the header promised {count}",
                    parts.len()
                )));
            }
            meta.chunk_frames = per;
            meta.chunk_count = count;
            note.push_str(&format!(
                ", in {count} chunks of {:.2} s ({} B of {} B here)",
                f64::from(per) * QOA_FRAME as f64 / f64::from(meta.sample_rate.max(1)),
                parts[0].len(),
                e.qoa.len()
            ));
            parts[0].clone()
        }
    };
    let mut w = BlobWriter::new(
        format::Sound::type_hash(),
        format::Sound::schema_version(),
        stamp,
    );
    w.table("meta", &[meta], row_hash::<SoundMeta>())
        .raw("qoa", qoa, 16);
    Ok((w.finish(), note))
}

/// Chunk `n` (1 and up) of a sound cooked in chunks.
pub fn cook_chunk(e: &Encoded, n: u32, stamp: u64) -> Result<Vec<u8>, CookError> {
    let (per, parts) = e
        .chunks
        .as_ref()
        .ok_or_else(|| CookError("not cooked in chunks".into()))?;
    let part = parts
        .get(n as usize)
        .filter(|_| n > 0)
        .ok_or_else(|| CookError(format!("no chunk {n} of {}", parts.len())))?;
    let total = u64::from(e.meta.frames).div_ceil(QOA_FRAME);
    let first = u64::from(*per) * u64::from(n);
    let meta = SoundChunkMeta {
        chunk: n,
        first_frame: first as u32,
        frame_count: (total - first).min(u64::from(*per)) as u32,
    };
    let mut w = BlobWriter::new(
        format::SoundChunk::type_hash(),
        format::SoundChunk::schema_version(),
        stamp,
    );
    w.table("meta", &[meta], row_hash::<SoundChunkMeta>())
        .raw("qoa", part.clone(), 16);
    Ok(w.finish())
}
