//! Sound files (WAV, MP3, Ogg Vorbis, FLAC, Ogg Opus) → 16-bit PCM → QOA
//! sound blobs, whole or in chunks, or Opus sound blobs (`sound { codec
//! opus }`: music and long sounds).
//!
//! An Opus sound (`og_bundle::opus`) is resampled to 48 kHz (a Kaiser
//! windowed sinc, built from `+ − × ÷` and `sqrt` so every host cooks the
//! same bytes), folded to its first two channels, and encoded in
//! self-contained blocks of 0.48 s; the cook decodes it back and notes the
//! round trip's signal-to-noise ratio. It is never cut into chunks: at
//! 96 kb/s a three-minute track is 2.4 MB, and its blocks are decoded as
//! the playheads reach them (the mixer's frame cache streams it).
//!
//! A long sound (`sound { stream true }`, or a QOA file above
//! [`STREAM_BYTES`] unless `stream false`) is cooked in chunks of whole QOA
//! frames, about `chunk` seconds each: the sound's blob holds the QOA
//! header and the first chunk, and every other chunk is a blob of its own
//! (`<id>#chunk<n>`, `og.SoundChunk`) that the runtime reads ahead of the
//! playhead and drops behind it. The file is encoded once and cut between
//! frames: a QOA frame carries its predictor's state, so the chunks decode
//! to exactly the samples of the whole file.

use super::{
    Action, CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, ext,
    settings_block, stamp,
};
use crate::format::{self, *};
use crate::qoa;
use og_blob::BlobWriter;

/// A QOA file above this many bytes is cooked in chunks unless its import
/// settings say `stream false` (1 MiB: 27 s of 48 kHz stereo).
pub const STREAM_BYTES: u64 = 1 << 20;

/// Samples per channel in a full QOA frame.
const QOA_FRAME: u64 = 5120;

/// The codec a sound is cooked to (`sound { codec … }`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Codec {
    /// QOA (the default): 3.2 bits per sample, decoded with integer
    /// arithmetic; every sound effect.
    #[default]
    Qoa,
    /// Opus at 48 kHz: music and long sounds, about a fifth of QOA's bytes.
    Opus,
}

/// `sound { … }` import settings of a sound file.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SoundSettings {
    /// Cooked in chunks: `Some(true)` always, `Some(false)` never, `None`
    /// above [`STREAM_BYTES`]. QOA only.
    pub stream: Option<bool>,
    /// Seconds per chunk (rounded up to whole QOA frames), 1..=60.
    pub chunk: f32,
    /// QOA or Opus.
    pub codec: Codec,
    /// An Opus sound's bitrate in kb/s, 16..=256; 0: 96 stereo, 64 mono.
    pub bitrate: u32,
}

impl Default for SoundSettings {
    fn default() -> Self {
        SoundSettings {
            stream: None,
            chunk: 4.0,
            codec: Codec::Qoa,
            bitrate: 0,
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
                "codec" => {
                    s.codec = match e.values.first().and_then(|v| v.as_word().or(v.as_str())) {
                        Some("qoa") => Codec::Qoa,
                        Some("opus") => Codec::Opus,
                        _ => {
                            return Err(format!(
                                "line {}: codec takes qoa (sound effects, the default) or opus (music and long sounds)",
                                e.line
                            ));
                        }
                    }
                }
                "bitrate" => {
                    let b = e.f32(0).map_err(|x| format!("line {}: {x}", e.line))?;
                    if !(16.0..=256.0).contains(&b) {
                        return Err(format!("line {}: bitrate must be 16..=256 kb/s", e.line));
                    }
                    s.bitrate = b.round() as u32;
                }
                k => {
                    return Err(format!(
                        "line {}: unknown sound setting '{k}' (stream, chunk, codec, bitrate)",
                        e.line
                    ));
                }
            }
        }
        if s.codec == Codec::Opus && s.stream == Some(true) {
            return Err(
                "an Opus sound is never cooked in chunks (`stream true`): its blocks are decoded as the playheads reach them; leave stream out, or use codec qoa"
                    .into(),
            );
        }
        Ok(s)
    }

    pub fn describe(&self) -> String {
        let mut d = format!(
            "stream {} chunk {}",
            self.stream.map_or("auto".to_string(), |b| b.to_string()),
            self.chunk
        );
        if self.codec == Codec::Opus {
            d.push_str(&format!(" codec opus bitrate {}", self.bitrate));
        }
        d
    }

    /// An Opus sound's bitrate in bits per second for `channels`.
    pub fn opus_bitrate(&self, channels: u32) -> u32 {
        match self.bitrate {
            0 if channels > 1 => 96_000,
            0 => 64_000,
            kb => kb * 1000,
        }
    }

    /// QOA frames per chunk and the chunk count for a sound of this shape,
    /// or `None` to cook it whole (always for Opus).
    pub fn chunking(&self, rate: u32, channels: u32, frames: u64) -> Option<(u32, u32)> {
        if self.codec == Codec::Opus {
            return None;
        }
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

/// The extensions `og cook` reads as sounds (besides an `.ogt` preset):
/// WAV, and MP3, Ogg Vorbis, FLAC and Ogg Opus decoded to PCM first.
pub const EXTENSIONS: &[&str] = &["wav", "mp3", "ogg", "flac", "opus"];

/// The format a sound file's extension names, for its messages.
fn format_name(name: &str) -> &'static str {
    match name
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("mp3") => "MP3",
        Some("ogg") => "Ogg Vorbis",
        Some("flac") => "FLAC",
        Some("opus") => "Ogg Opus",
        _ => "WAV",
    }
}

/// A sound file decoded: interleaved 16-bit samples.
#[derive(Debug)]
pub struct Pcm {
    pub samples: Vec<i16>,
    pub rate: u32,
    pub channels: u32,
    /// What it was decoded from (`16-bit PCM`, `MP3`), for the note.
    pub from: String,
}

impl Pcm {
    /// Samples per channel.
    pub fn frames(&self) -> u64 {
        (self.samples.len() / self.channels.max(1) as usize) as u64
    }
}

/// The shape of a sound file: rate, channels, samples per channel. A WAV's
/// comes from its header; a compressed file is decoded, since only its
/// decoded length is exact.
pub fn shape(name: &str, bytes: &[u8]) -> Result<(u32, u32, u64), String> {
    if format_name(name) != "WAV" {
        let p = decode(name, bytes)?;
        return Ok((p.rate, p.channels, p.frames()));
    }
    let r = hound::WavReader::new(std::io::Cursor::new(bytes))
        .map_err(|e| format!("cannot decode {name} as WAV: {e}"))?;
    let spec = r.spec();
    Ok((
        spec.sample_rate,
        u32::from(spec.channels),
        u64::from(r.duration()),
    ))
}

/// Decodes a sound file by its extension (`EXTENSIONS`) to 16-bit PCM.
pub fn decode(name: &str, bytes: &[u8]) -> Result<Pcm, String> {
    match format_name(name) {
        "WAV" => wav(name, bytes),
        "Ogg Opus" => ogg_opus(name, bytes),
        // An `.ogg` may hold Opus rather than Vorbis.
        "Ogg Vorbis" if is_ogg_opus(bytes) => ogg_opus(name, bytes),
        f => compressed(name, f, bytes),
    }
}

/// Whether an Ogg file's first packet is an `OpusHead`.
fn is_ogg_opus(bytes: &[u8]) -> bool {
    bytes.starts_with(b"OggS") && bytes.windows(8).take(128).any(|w| w == b"OpusHead")
}

/// An Ogg Opus file (RFC 7845) as 16-bit PCM at 48 kHz: its header's
/// output gain applied, the encoder delay and the final granule's padding
/// trimmed (the opus-pure codec og-bundle's `opus` encodes with).
fn ogg_opus(name: &str, bytes: &[u8]) -> Result<Pcm, String> {
    use opus_pure::{MAX_PACKET_SAMPLES, OggOpusReader, Trim};
    let fail = |e: &dyn std::fmt::Display| format!("cannot decode {name} as Ogg Opus: {e}");
    let mut reader = OggOpusReader::new(std::io::Cursor::new(bytes)).map_err(|e| fail(&e))?;
    let head = reader.head().clone();
    let channels = usize::from(head.channel_count);
    let mut decoder = head.decoder(48_000).map_err(|e| fail(&e))?;
    let mut trim = Trim::new(&head, 48_000, channels).map_err(|e| fail(&e))?;
    let mut block = vec![0i16; MAX_PACKET_SAMPLES * channels];
    let mut samples = Vec::new();
    for packet in reader.packets() {
        let packet = packet.map_err(|e| fail(&e))?;
        let n = decoder
            .decode_s16(&packet.data, MAX_PACKET_SAMPLES, &mut block)
            .map_err(|e| fail(&e))?;
        samples.extend_from_slice(trim.keep(&packet, &block[..n * channels]));
    }
    if samples.is_empty() || channels == 0 {
        return Err(fail(&"no audio decoded"));
    }
    Ok(Pcm {
        samples,
        rate: 48_000,
        channels: channels as u32,
        from: "Ogg Opus".into(),
    })
}

/// A WAV file (8/16/24/32-bit integer or 32-bit float PCM, 1 to 8
/// channels) as 16-bit PCM.
pub fn wav(name: &str, bytes: &[u8]) -> Result<Pcm, String> {
    let fail = |e: hound::Error| format!("cannot decode {name} as WAV: {e}");
    let reader = hound::WavReader::new(std::io::Cursor::new(bytes)).map_err(fail)?;
    let spec = reader.spec();
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
            return Err(format!(
                "cannot decode {name} as WAV: {b}-bit {f:?} samples are not supported"
            ));
        }
    }
    .map_err(fail)?;
    Ok(Pcm {
        samples,
        rate: spec.sample_rate,
        channels: u32::from(spec.channels),
        from: format!("{}-bit PCM", spec.bits_per_sample),
    })
}

/// An MP3, Ogg Vorbis or FLAC file (symphonia) as 16-bit PCM: its first
/// audio track, encoder delay and padding trimmed.
fn compressed(name: &str, format: &str, bytes: &[u8]) -> Result<Pcm, String> {
    use symphonia::core::codecs::CodecParameters;
    use symphonia::core::codecs::audio::AudioDecoderOptions;
    use symphonia::core::errors::Error;
    use symphonia::core::formats::probe::Hint;
    use symphonia::core::formats::{FormatOptions, TrackType};
    use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
    use symphonia::core::meta::MetadataOptions;

    let fail = |e: &dyn std::fmt::Display| format!("cannot decode {name} as {format}: {e}");
    let mut hint = Hint::new();
    if let Some((_, e)) = name.rsplit_once('.') {
        hint.with_extension(e);
    }
    let source = MediaSourceStream::new(
        Box::new(std::io::Cursor::new(bytes)),
        MediaSourceStreamOptions::default(),
    );
    let mut reader = symphonia::default::get_probe()
        .probe(
            &hint,
            source,
            FormatOptions::default(),
            MetadataOptions::default(),
        )
        .map_err(|e| fail(&e))?;
    let track = reader
        .default_track(TrackType::Audio)
        .ok_or_else(|| fail(&"no audio track"))?;
    let id = track.id;
    let Some(CodecParameters::Audio(params)) = track.codec_params.clone() else {
        return Err(fail(&"no audio track"));
    };
    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(&params, &AudioDecoderOptions::default())
        .map_err(|e| fail(&e))?;
    let (mut rate, mut channels) = (
        params.sample_rate.unwrap_or(0),
        params.channels.as_ref().map_or(0, |c| c.count() as u32),
    );
    let mut samples: Vec<i16> = Vec::new();
    let mut packet_samples: Vec<i16> = Vec::new();
    loop {
        let packet = match reader.next_packet() {
            Ok(Some(p)) => p,
            Ok(None) => break,
            // A file cut short ends where it was cut.
            Err(Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(fail(&e)),
        };
        if packet.track_id != id {
            continue;
        }
        let buf = match decoder.decode(&packet) {
            Ok(b) => b,
            // A corrupt packet is skipped, as players do.
            Err(Error::DecodeError(_)) => continue,
            Err(e) => return Err(fail(&e)),
        };
        let spec = buf.spec();
        let (r, c) = (spec.rate(), spec.channels().count() as u32);
        if samples.is_empty() {
            (rate, channels) = (r, c);
        } else if (r, c) != (rate, channels) {
            return Err(fail(&format!(
                "the stream changes from {rate} Hz, {channels} channel(s) to {r} Hz, {c}"
            )));
        }
        buf.copy_to_vec_interleaved(&mut packet_samples);
        samples.extend_from_slice(&packet_samples);
    }
    if samples.is_empty() || rate == 0 || channels == 0 {
        return Err(fail(&"no audio decoded"));
    }
    Ok(Pcm {
        samples,
        rate,
        channels,
        from: format.to_string(),
    })
}

/// A sound encoded to QOA (or Opus), once per cook (its chunks share it).
#[derive(Debug)]
pub struct Encoded {
    /// The QOA file (empty for Opus).
    pub qoa: Vec<u8>,
    /// The Opus sound (`meta.codec` Opus; empty for QOA).
    pub opus: Vec<u8>,
    pub meta: SoundMeta,
    pub note: String,
    /// Cooked in chunks: (QOA frames per chunk, the chunks as
    /// `qoa::split` cuts them).
    pub chunks: Option<(u32, Vec<Vec<u8>>)>,
}

/// Encodes decoded PCM to QOA, cut into chunks of `per_chunk` QOA frames
/// when given.
pub fn encode(pcm: &Pcm, per_chunk: Option<u32>) -> Result<Encoded, String> {
    let channels = pcm.channels as usize;
    let qoa = qoa::encode(&pcm.samples, channels, pcm.rate)?;
    let frames = pcm.frames() as u32;
    let chunks = match per_chunk {
        Some(per) => Some((per, qoa::split(&qoa, per as usize)?)),
        None => None,
    };
    Ok(Encoded {
        chunks,
        qoa,
        opus: Vec::new(),
        meta: SoundMeta {
            sample_rate: pcm.rate,
            channels: pcm.channels,
            frames,
            ..SoundMeta::default()
        },
        note: format!(
            "{} Hz, {channels} channel(s), {:.2} s, {} to QOA",
            pcm.rate,
            frames as f64 / pcm.rate.max(1) as f64,
            pcm.from
        ),
    })
}

/// Encodes decoded PCM to an Opus sound at `bitrate` bits per second:
/// resampled to 48 kHz, its first two channels, then decoded back to
/// check the round trip (the note gives its signal-to-noise ratio).
pub fn encode_opus(pcm: &Pcm, bitrate: u32) -> Result<Encoded, String> {
    let channels = (pcm.channels as usize).clamp(1, 2);
    let from = pcm.channels.max(1) as usize;
    let folded: Vec<i16> = if from == channels {
        pcm.samples.clone()
    } else {
        pcm.samples
            .chunks_exact(from)
            .flat_map(|f| f[..channels].to_vec())
            .collect()
    };
    let at48 = resample(&folded, channels, pcm.rate, crate::opus::RATE);
    let opus = crate::opus::encode(&at48, channels, bitrate, pcm.rate)?;
    let back = crate::opus::decode(&opus)?;
    if back.samples.len() != at48.len() {
        return Err(format!(
            "the Opus round trip gave {} samples for {}",
            back.samples.len(),
            at48.len()
        ));
    }
    let (signal, noise) = at48
        .iter()
        .zip(&back.samples)
        .fold((0f64, 0f64), |(s, n), (&a, &b)| {
            let (a, b) = (f64::from(a), f64::from(b));
            (s + a * a, n + (a - b) * (a - b))
        });
    let snr = if noise > 0.0 {
        format!("{:.1} dB", 10.0 * (signal / noise).log10())
    } else {
        "exact".into()
    };
    let frames = (at48.len() / channels) as u32;
    let resampled = if pcm.rate == crate::opus::RATE {
        String::new()
    } else {
        format!(" (from {} Hz)", pcm.rate)
    };
    Ok(Encoded {
        qoa: Vec::new(),
        meta: SoundMeta {
            sample_rate: crate::opus::RATE,
            channels: channels as u32,
            frames,
            codec: sound_codec::OPUS,
            ..SoundMeta::default()
        },
        note: format!(
            "48000 Hz{resampled}, {channels} channel(s), {:.2} s, {} to Opus at {} kb/s ({} B, {} blocks; round trip SNR {snr})",
            f64::from(frames) / f64::from(crate::opus::RATE),
            pcm.from,
            bitrate / 1000,
            opus.len(),
            crate::opus::block_count(u64::from(frames)),
        ),
        opus,
        chunks: None,
    })
}

/// sin(πt), from a Taylor series after reducing t to [-½, ½] (no libm, so
/// the resampler's kernel is the same on every host).
fn sin_pi(t: f64) -> f64 {
    let n = t.round();
    let x = std::f64::consts::PI * (t - n);
    let x2 = x * x;
    let (mut term, mut sum) = (x, x);
    for k in 1..14 {
        term *= -x2 / ((2 * k) as f64 * (2 * k + 1) as f64);
        sum += term;
    }
    if (n as i64) % 2 == 0 { sum } else { -sum }
}

/// The modified Bessel function I0, by its power series.
fn bessel_i0(x: f64) -> f64 {
    let (mut term, mut sum) = (1.0f64, 1.0f64);
    let q = x * x / 4.0;
    for k in 1..80 {
        term *= q / (k * k) as f64;
        sum += term;
        if term < sum * 1e-17 {
            break;
        }
    }
    sum
}

/// Resamples interleaved PCM from `from` Hz to `to` Hz: a polyphase
/// Kaiser-windowed sinc (β 8.6, 16 zero crossings each side of the lower
/// rate's Nyquist, about 90 dB of stopband), each phase normalised so DC
/// passes exactly. Rational: output frame `n` is input position
/// `n · from / to`, every phase from one table.
pub fn resample(samples: &[i16], channels: usize, from: u32, to: u32) -> Vec<i16> {
    if from == to || from == 0 || to == 0 || samples.is_empty() {
        return samples.to_vec();
    }
    let gcd = |mut a: u64, mut b: u64| {
        while b != 0 {
            (a, b) = (b, a % b);
        }
        a
    };
    let g = gcd(u64::from(from), u64::from(to));
    let (up, down) = (u64::from(to) / g, u64::from(from) / g);
    const ZEROS: f64 = 16.0;
    const BETA: f64 = 8.6;
    // The cutoff relative to the input's Nyquist (below 1 when downsampling).
    let cutoff = (f64::from(to) / f64::from(from)).min(1.0);
    let reach = (ZEROS / cutoff).ceil() as usize;
    let taps = 2 * reach;
    let i0_beta = bessel_i0(BETA);
    let mut table = vec![0f64; up as usize * taps];
    for p in 0..up as usize {
        let frac = p as f64 / up as f64;
        let row = &mut table[p * taps..(p + 1) * taps];
        for (j, w) in row.iter_mut().enumerate() {
            let x = j as f64 - (reach - 1) as f64 - frac;
            let r = x / reach as f64;
            let window = bessel_i0(BETA * (1.0 - r * r).max(0.0).sqrt()) / i0_beta;
            let t = x * cutoff;
            let sinc = if t == 0.0 {
                1.0
            } else {
                sin_pi(t) / (std::f64::consts::PI * t)
            };
            *w = sinc * window;
        }
        let sum: f64 = row.iter().sum();
        row.iter_mut().for_each(|w| *w /= sum);
    }
    let frames = (samples.len() / channels) as u64;
    let out_frames = (frames * up).div_ceil(down);
    let mut out = Vec::with_capacity(out_frames as usize * channels);
    for n in 0..out_frames {
        let pos = n * down;
        let (i, p) = ((pos / up) as i64, (pos % up) as usize);
        let row = &table[p * taps..(p + 1) * taps];
        for c in 0..channels {
            let mut acc = 0f64;
            for (j, w) in row.iter().enumerate() {
                let k = i - (reach as i64 - 1) + j as i64;
                if k >= 0 && (k as u64) < frames {
                    acc += w * f64::from(samples[k as usize * channels + c]);
                }
            }
            out.push(acc.round().clamp(-32768.0, 32767.0) as i16);
        }
    }
    out
}

/// The sound's blob: the whole QOA file, or with `chunking` (QOA frames
/// per chunk, chunks) its header and first chunk; an Opus sound whole.
pub fn cook(
    e: &Encoded,
    chunking: Option<(u32, u32)>,
    stamp: u64,
) -> Result<(Vec<u8>, String), CookError> {
    let mut meta = e.meta;
    let mut note = e.note.clone();
    if meta.codec == sound_codec::OPUS {
        if chunking.is_some() {
            return Err(CookError("an Opus sound is never cooked in chunks".into()));
        }
        let mut w = BlobWriter::new(
            format::Sound::type_hash(),
            format::Sound::schema_version(),
            stamp,
        );
        w.table("meta", &[meta], row_hash::<SoundMeta>())
            .raw("opus", e.opus.clone(), 16);
        return Ok((w.finish(), note));
    }
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

/// A sound file: cooked whole, or its first chunk with the rest after it.
pub(crate) static SOUND: KindDef = KindDef {
    name: "sound",
    extensions: EXTENSIONS,
    types: &[],
    by_content: true,
    streams: true,
    plan: plan_sound,
};

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&SOUND];

fn plan_sound<'a>(
    _cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let PlanState { early_errors, .. } = state;
    let settings = match SoundSettings::parse(settings_block(s, "sound")) {
        Ok(x) => x,
        Err(e) => {
            early_errors.push((s.rel.clone(), "sound", e));
            return;
        }
    };
    // An unreadable file fails in the cook, with its reason.
    let chunking = shape(&s.rel, &s.bytes)
        .ok()
        .and_then(|(rate, ch, frames)| settings.chunking(rate, ch, frames));
    let shape = chunking.map_or("whole".to_string(), |(p, n)| format!("{n} chunks of {p}"));
    let st = stamp(&[
        b"sound",
        &hash,
        settings.describe().as_bytes(),
        shape.as_bytes(),
    ]);
    outs.push((
        s.rel.clone(),
        format::Sound::NAME,
        st,
        Box::new(SoundJob(s, chunking)),
    ));
    if let Some((per, count)) = chunking {
        for n in 1..count {
            let st = stamp(&[b"sound chunk", &hash, shape.as_bytes(), &n.to_le_bytes()]);
            outs.push((
                format!("{}#chunk{n}", s.rel),
                format::SoundChunk::NAME,
                st,
                Box::new(SoundChunkJob(s, per, n)),
            ));
        }
    }
}

/// A sound's output: the whole sound, or its first chunk of so many QOA
/// frames.
struct SoundJob<'a>(&'a Source, Option<(u32, u32)>);

impl Job for SoundJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let SoundJob(s, chunking) = self;
        let e = s.encoded(chunking.map(|c| c.0))?;
        cook(&e, *chunking, p.stamp).map(|(b, n)| (b, vec![n]))
    }

    fn action(&self) -> Action<'_> {
        let SoundJob(s, chunks) = self;
        Action::Input(
            s.bytes.as_slice(),
            format!("sound {chunks:?} {} {:?}", ext(&s.rel), s.settings),
            false,
        )
    }
}

/// Chunk `n` (1 and up) of a sound cooked in chunks of so many QOA frames.
struct SoundChunkJob<'a>(&'a Source, u32, u32);

impl Job for SoundChunkJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let SoundChunkJob(s, per, n) = self;
        let e = s.encoded(Some(*per))?;
        cook_chunk(&e, *n, p.stamp).map(|b| (b, Vec::new()))
    }

    fn action(&self) -> Action<'_> {
        let SoundChunkJob(s, chunk, count) = self;
        Action::Input(
            s.bytes.as_slice(),
            format!(
                "sound-chunk {chunk} {count} {} {:?}",
                ext(&s.rel),
                s.settings
            ),
            false,
        )
    }
}
