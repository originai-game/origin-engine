//! The engine's Opus sounds: Opus (RFC 6716) packets in self-contained
//! blocks, so a long sound decodes from anywhere and streams through the
//! same frame cache as QOA. Music and long sounds cook to it
//! (`sound { codec opus }`): about a fifth of QOA's bytes at 96 kb/s.
//!
//! **Blocks.** The sound is cut into blocks of [`BLOCK_FRAMES`] samples per
//! channel (24 packets of 20 ms at 48 kHz: 0.48 s). Each block is its own
//! Opus stream: the cook starts a fresh encoder [`PREROLL_PACKETS`] packets
//! (40 ms) before the block and keeps every packet it makes, so decoding a
//! block is a fresh decoder over those packets, the pre-roll's output
//! discarded. A block therefore decodes to the same samples whoever decodes
//! it and whatever was decoded before (a voice starting half way into a
//! track, a loop's wrap, a decode-ahead helper): the property QOA frames
//! have, which the mixer's frame cache is built on. The seams cost nothing
//! audible: the encoder and decoder start together, so each block is
//! coded as well as one continuous stream would be (measured on tones and
//! noise at 96 kb/s: the same SNR, and no step at a seam larger than the
//! signal's own), for about a tenth more bytes than one stream.
//!
//! **Encoding** is CELT only (`Application::RestrictedLowDelay`: no mode
//! switches between speech and music coding at block starts, 2.5 ms of
//! encoder delay, which the input is advanced by so the output lines up
//! with the source sample for sample), at 48 kHz, mono or stereo. The cook
//! resamples other rates to 48 kHz and folds more channels to two first.
//!
//! File (little-endian):
//!
//! | offset | type | field |
//! |---:|---|---|
//! | 0 | u32 | magic `ogop` |
//! | 4 | u8 | version (1) |
//! | 5 | u8 | channels (1 or 2) |
//! | 6 | u16 | packets per block ([`BLOCK_PACKETS`]) |
//! | 8 | u16 | pre-roll packets per block ([`PREROLL_PACKETS`]) |
//! | 10 | u16 | samples per packet ([`PACKET_FRAMES`]) |
//! | 12 | u32 | samples per channel at 48 kHz |
//! | 16 | u32 | the source's rate before resampling (informational) |
//! | 20 | u32 | the bitrate it was encoded at, bits per second |
//! | 24 | u32 | blocks |
//! | 28 | u32 × (blocks + 1) | each block's byte offset from the file's start, then the file's end |
//!
//! then the blocks, each its packets (pre-roll first) as a u16 byte length
//! and the packet.

use opus_pure::{Application, OpusDecoder, OpusEncoder};

/// "ogop", little-endian.
pub const MAGIC: u32 = u32::from_le_bytes(*b"ogop");
const VERSION: u8 = 1;
/// The rate every Opus sound decodes at.
pub const RATE: u32 = 48_000;
/// Samples per channel in one packet (20 ms).
pub const PACKET_FRAMES: usize = 960;
/// Packets per block.
pub const BLOCK_PACKETS: usize = 24;
/// Packets coded before each block and decoded but not heard.
pub const PREROLL_PACKETS: usize = 2;
/// Samples per channel in a full block (0.48 s).
pub const BLOCK_FRAMES: u64 = (BLOCK_PACKETS * PACKET_FRAMES) as u64;
/// The bytes before the block table.
const HEADER: usize = 28;
/// The largest packet Opus makes.
const MAX_PACKET: usize = 1275;

/// What an Opus sound's header says, and where each block's bytes are.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Index {
    pub channels: usize,
    /// Samples per channel at [`RATE`].
    pub frames: u64,
    /// The source's rate before the cook resampled it.
    pub source_rate: u32,
    /// Bits per second it was encoded at.
    pub bitrate: u32,
    /// Each block's byte range in the file.
    pub blocks: Vec<(usize, usize)>,
}

impl Index {
    /// Samples per channel in block `k`.
    pub fn block_len(&self, k: usize) -> u64 {
        self.frames
            .saturating_sub(k as u64 * BLOCK_FRAMES)
            .min(BLOCK_FRAMES)
    }
}

/// Blocks a sound of `frames` samples per channel takes (at least one).
pub fn block_count(frames: u64) -> usize {
    frames.div_ceil(BLOCK_FRAMES).max(1) as usize
}

/// Encodes interleaved 16-bit PCM at 48 kHz (1 or 2 channels) at `bitrate`
/// bits per second; `source_rate` is recorded for the cook's notes.
pub fn encode(
    samples: &[i16],
    channels: usize,
    bitrate: u32,
    source_rate: u32,
) -> Result<Vec<u8>, String> {
    if !(1..=2).contains(&channels) {
        return Err(format!(
            "Opus sounds are mono or stereo, not {channels} channels"
        ));
    }
    if !samples.len().is_multiple_of(channels) {
        return Err("Opus encode: the samples are not whole frames".into());
    }
    let frames = (samples.len() / channels) as u64;
    if frames > u64::from(u32::MAX) {
        return Err("Opus encode: longer than 2^32 samples".into());
    }
    let blocks = block_count(frames);
    let mut body: Vec<u8> = Vec::new();
    let mut offsets = Vec::with_capacity(blocks + 1);
    let table = HEADER + 4 * (blocks + 1);
    let at = |i: i64, c: usize| -> i16 {
        if i < 0 || i as u64 >= frames {
            0
        } else {
            samples[i as usize * channels + c]
        }
    };
    let mut input = vec![0i16; PACKET_FRAMES * channels];
    let mut packet = vec![0u8; MAX_PACKET + 1];
    for k in 0..blocks {
        offsets.push((table + body.len()) as u32);
        let mut enc = OpusEncoder::new(RATE as i32, channels, Application::RestrictedLowDelay)
            .map_err(|e| format!("Opus encoder: {e}"))?;
        enc.bitrate_bps = bitrate.clamp(6_000, 510_000) as i32;
        enc.complexity = 10;
        // The encoder's output lags its input by its lookahead: feed it
        // that far ahead, so the decoded block lines up with the source.
        let lead = enc.lookahead() as i64;
        let len = (frames - k as u64 * BLOCK_FRAMES).min(BLOCK_FRAMES) as usize;
        let packets = PREROLL_PACKETS + len.div_ceil(PACKET_FRAMES);
        let first = (k as u64 * BLOCK_FRAMES) as i64 - (PREROLL_PACKETS * PACKET_FRAMES) as i64;
        for p in 0..packets {
            let base = first + (p * PACKET_FRAMES) as i64 + lead;
            for j in 0..PACKET_FRAMES {
                for c in 0..channels {
                    input[j * channels + c] = at(base + j as i64, c);
                }
            }
            let n = enc
                .encode_s16(&input, PACKET_FRAMES, &mut packet)
                .map_err(|e| format!("Opus encode: {e}"))?;
            body.extend_from_slice(&(n as u16).to_le_bytes());
            body.extend_from_slice(&packet[..n]);
        }
    }
    offsets.push((table + body.len()) as u32);
    let mut out = Vec::with_capacity(table + body.len());
    out.extend_from_slice(&MAGIC.to_le_bytes());
    out.push(VERSION);
    out.push(channels as u8);
    out.extend_from_slice(&(BLOCK_PACKETS as u16).to_le_bytes());
    out.extend_from_slice(&(PREROLL_PACKETS as u16).to_le_bytes());
    out.extend_from_slice(&(PACKET_FRAMES as u16).to_le_bytes());
    out.extend_from_slice(&(frames as u32).to_le_bytes());
    out.extend_from_slice(&source_rate.to_le_bytes());
    out.extend_from_slice(&bitrate.to_le_bytes());
    out.extend_from_slice(&(blocks as u32).to_le_bytes());
    for o in offsets {
        out.extend_from_slice(&o.to_le_bytes());
    }
    out.extend_from_slice(&body);
    Ok(out)
}

/// Whether `bytes` start like an Opus sound.
pub fn is_opus(bytes: &[u8]) -> bool {
    bytes.get(0..4) == Some(&MAGIC.to_le_bytes()[..])
}

/// Reads the header and block table, checking them against the file.
pub fn index(bytes: &[u8]) -> Result<Index, String> {
    let u16_at = |o: usize| -> Option<u16> {
        Some(u16::from_le_bytes(bytes.get(o..o + 2)?.try_into().ok()?))
    };
    let u32_at = |o: usize| -> Option<u32> {
        Some(u32::from_le_bytes(bytes.get(o..o + 4)?.try_into().ok()?))
    };
    let bad = |what: &str| format!("malformed Opus sound: {what}");
    if bytes.len() < HEADER || !is_opus(bytes) {
        return Err(bad("no header"));
    }
    if bytes[4] != VERSION {
        return Err(bad(&format!(
            "version {} (this engine reads {VERSION})",
            bytes[4]
        )));
    }
    let channels = usize::from(bytes[5]);
    let shape = (
        usize::from(u16_at(6).unwrap_or(0)),
        usize::from(u16_at(8).unwrap_or(0)),
        usize::from(u16_at(10).unwrap_or(0)),
    );
    if !(1..=2).contains(&channels) || shape != (BLOCK_PACKETS, PREROLL_PACKETS, PACKET_FRAMES) {
        return Err(bad("unsupported channels or block shape"));
    }
    let frames = u64::from(u32_at(12).ok_or_else(|| bad("no length"))?);
    let source_rate = u32_at(16).unwrap_or(0);
    let bitrate = u32_at(20).unwrap_or(0);
    let count = u32_at(24).ok_or_else(|| bad("no block count"))? as usize;
    if count != block_count(frames) {
        return Err(bad(&format!("{count} blocks for {frames} samples")));
    }
    let table = HEADER + 4 * (count + 1);
    let mut blocks = Vec::with_capacity(count);
    for k in 0..count {
        let a = u32_at(HEADER + 4 * k).ok_or_else(|| bad("truncated block table"))? as usize;
        let b = u32_at(HEADER + 4 * (k + 1)).ok_or_else(|| bad("truncated block table"))? as usize;
        if a < table || b < a || b > bytes.len() {
            return Err(bad(&format!("block {k} lies outside the file")));
        }
        blocks.push((a, b));
    }
    if blocks.last().is_some_and(|&(_, end)| end != bytes.len()) {
        return Err(bad("bytes past the last block"));
    }
    Ok(Index {
        channels,
        frames,
        source_rate,
        bitrate,
        blocks,
    })
}

/// Decodes one block's bytes (its packets, pre-roll first) to `len`
/// samples per channel, interleaved: a fresh decoder, the pre-roll's output
/// dropped. The same bytes give the same samples on a host.
pub fn decode_block(block: &[u8], channels: usize, len: u64) -> Result<Vec<i16>, String> {
    let mut d = BlockDecoder::new(channels, len)?;
    let mut rest = block;
    loop {
        match d.step(&mut rest, usize::MAX)? {
            Some(samples) => return Ok(samples),
            None if rest.is_empty() => return Err("malformed Opus block: too few packets".into()),
            None => {}
        }
    }
}

/// A block decode that can stop between packets and go on later (a host
/// spreading its decoding over frames): [`decode_block`] in steps, to the
/// same samples.
pub struct BlockDecoder {
    decoder: OpusDecoder,
    channels: usize,
    len: usize,
    /// Packets decoded so far, the pre-roll included.
    packets: usize,
    out: Vec<i16>,
    scratch: Vec<i16>,
}

impl BlockDecoder {
    /// A decoder for a block of `len` samples per channel.
    pub fn new(channels: usize, len: u64) -> Result<BlockDecoder, String> {
        Ok(BlockDecoder {
            decoder: OpusDecoder::new(RATE as i32, channels)
                .map_err(|e| format!("Opus decoder: {e}"))?,
            channels,
            len: len as usize,
            packets: 0,
            out: Vec::with_capacity(len as usize * channels),
            scratch: vec![0; PACKET_FRAMES * channels],
        })
    }

    /// Packets decoded so far, the pre-roll included.
    pub fn packets(&self) -> usize {
        self.packets
    }

    /// Decodes up to `max` packets from the front of `rest` (advancing it);
    /// the block's samples once it is whole.
    pub fn step(&mut self, rest: &mut &[u8], max: usize) -> Result<Option<Vec<i16>>, String> {
        let want = self.len * self.channels;
        let mut n = 0;
        while self.out.len() < want && n < max {
            let Some(size) = rest
                .get(0..2)
                .map(|b| usize::from(u16::from_le_bytes([b[0], b[1]])))
            else {
                return Ok(None);
            };
            let packet = rest
                .get(2..2 + size)
                .ok_or("malformed Opus block: a packet runs past its block")?;
            *rest = &rest[2 + size..];
            let got = self
                .decoder
                .decode_s16(packet, PACKET_FRAMES, &mut self.scratch)
                .map_err(|e| format!("Opus decode: {e}"))?;
            if self.packets >= PREROLL_PACKETS {
                let take = (got * self.channels).min(want - self.out.len());
                self.out.extend_from_slice(&self.scratch[..take]);
            }
            self.packets += 1;
            n += 1;
        }
        Ok((self.out.len() >= want).then(|| std::mem::take(&mut self.out)))
    }
}

/// A whole Opus sound decoded.
#[derive(Clone, Debug, PartialEq)]
pub struct Decoded {
    pub channels: usize,
    /// Always [`RATE`].
    pub sample_rate: u32,
    /// Interleaved, `frames × channels`.
    pub samples: Vec<i16>,
}

/// Decodes a whole Opus sound, block after block.
pub fn decode(bytes: &[u8]) -> Result<Decoded, String> {
    let ix = index(bytes)?;
    let mut samples = Vec::with_capacity(ix.frames as usize * ix.channels);
    for (k, &(a, b)) in ix.blocks.iter().enumerate() {
        samples.extend(decode_block(&bytes[a..b], ix.channels, ix.block_len(k))?);
    }
    Ok(Decoded {
        channels: ix.channels,
        sample_rate: RATE,
        samples,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(frames: usize, channels: usize) -> Vec<i16> {
        (0..frames * channels)
            .map(|i| {
                let (f, c) = ((i / channels) as f64, (i % channels) as f64);
                let t = f / f64::from(RATE);
                (12_000.0 * (std::f64::consts::TAU * (440.0 + 220.0 * c) * t).sin()) as i16
            })
            .collect()
    }

    #[test]
    fn blocks_index_and_decode_to_the_sound_s_length() {
        for (frames, channels) in [(1usize, 1usize), (23_040, 2), (23_041, 1), (100_000, 2)] {
            let pcm = tone(frames, channels);
            let bytes = encode(&pcm, channels, 96_000, 44_100).unwrap();
            let ix = index(&bytes).unwrap();
            assert_eq!(
                (ix.channels, ix.frames, ix.source_rate),
                (channels, frames as u64, 44_100)
            );
            assert_eq!(ix.blocks.len(), block_count(frames as u64));
            let d = decode(&bytes).unwrap();
            assert_eq!(d.samples.len(), frames * channels);
        }
    }

    #[test]
    fn a_block_decoded_in_steps_is_the_block_decoded_at_once() {
        let pcm = tone(60_000, 2);
        let bytes = encode(&pcm, 2, 64_000, 48_000).unwrap();
        let ix = index(&bytes).unwrap();
        let (a, b) = ix.blocks[1];
        let whole = decode_block(&bytes[a..b], 2, ix.block_len(1)).unwrap();
        let mut d = BlockDecoder::new(2, ix.block_len(1)).unwrap();
        let mut rest = &bytes[a..b];
        let stepped = loop {
            if let Some(s) = d.step(&mut rest, 3).unwrap() {
                break s;
            }
        };
        assert_eq!(whole, stepped);
    }

    #[test]
    fn malformed_files_are_refused() {
        let bytes = encode(&tone(30_000, 1), 1, 64_000, 48_000).unwrap();
        assert!(index(&bytes[..bytes.len() - 1]).is_err());
        let mut wrong = bytes.clone();
        wrong[24] = 9;
        assert!(index(&wrong).is_err());
        assert!(index(b"qoaf0000").is_err());
        assert!(encode(&[0; 9], 3, 64_000, 48_000).is_err());
    }
}
