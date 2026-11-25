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

/// Encodes a single Opus block `k` given a sample lookup function `at(sample_idx, channel)`.
fn encode_one_block(
    channels: usize,
    bitrate: u32,
    k: usize,
    frames: u64,
    at: impl Fn(i64, usize) -> i16,
) -> Result<Vec<u8>, String> {
    let mut enc = OpusEncoder::new(RATE as i32, channels, Application::RestrictedLowDelay)
        .map_err(|e| format!("Opus encoder: {e}"))?;
    enc.bitrate_bps = bitrate.clamp(6_000, 510_000) as i32;
    enc.complexity = 10;
    let lead = enc.lookahead() as i64;
    let len = (frames - k as u64 * BLOCK_FRAMES).min(BLOCK_FRAMES) as usize;
    let packets = PREROLL_PACKETS + len.div_ceil(PACKET_FRAMES);
    let first = (k as u64 * BLOCK_FRAMES) as i64 - (PREROLL_PACKETS * PACKET_FRAMES) as i64;
    let mut input = vec![0i16; PACKET_FRAMES * channels];
    let mut packet = vec![0u8; MAX_PACKET + 1];
    let mut body = Vec::new();
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
    Ok(body)
}

/// Writes an `ogop` header + block table + blocks. `total_frames` is the
/// full sound's length in samples per channel; `blocks` holds either every
/// block (whole sound) or chunk 0's blocks (chunked sound head).
pub fn encode_blocks_file(
    blocks: &[Vec<u8>],
    channels: usize,
    total_frames: u64,
    source_rate: u32,
    bitrate: u32,
) -> Result<Vec<u8>, String> {
    if !(1..=2).contains(&channels) {
        return Err(format!(
            "Opus sounds are mono or stereo, not {channels} channels"
        ));
    }
    if total_frames > u64::from(u32::MAX) {
        return Err("Opus encode: longer than 2^32 samples".into());
    }
    if blocks.is_empty() {
        return Err("Opus encode: at least one block is required".into());
    }
    let count = blocks.len();
    let table = HEADER + 4 * (count + 1);
    let body_len: usize = blocks.iter().map(Vec::len).sum();
    let mut out = Vec::with_capacity(table + body_len);
    out.extend_from_slice(&MAGIC.to_le_bytes());
    out.push(VERSION);
    out.push(channels as u8);
    out.extend_from_slice(&(BLOCK_PACKETS as u16).to_le_bytes());
    out.extend_from_slice(&(PREROLL_PACKETS as u16).to_le_bytes());
    out.extend_from_slice(&(PACKET_FRAMES as u16).to_le_bytes());
    out.extend_from_slice(&(total_frames as u32).to_le_bytes());
    out.extend_from_slice(&source_rate.to_le_bytes());
    out.extend_from_slice(&bitrate.to_le_bytes());
    out.extend_from_slice(&(count as u32).to_le_bytes());
    let mut off = table as u32;
    for b in blocks {
        out.extend_from_slice(&off.to_le_bytes());
        off = off
            .checked_add(b.len() as u32)
            .ok_or("Opus encode: file exceeds 4 GiB")?;
    }
    out.extend_from_slice(&off.to_le_bytes());
    for b in blocks {
        out.extend_from_slice(b);
    }
    Ok(out)
}

/// Encodes one chunk (`n >= 1`) of Opus blocks: `u32 block_count`,
/// `u32 offsets[block_count + 1]` from the chunk's start, then the blocks.
pub fn encode_chunk(blocks: &[Vec<u8>]) -> Result<Vec<u8>, String> {
    if blocks.is_empty() {
        return Err("Opus chunk: empty block slice".into());
    }
    let count = blocks.len();
    let table = 4 + 4 * (count + 1);
    let body_len: usize = blocks.iter().map(Vec::len).sum();
    let mut out = Vec::with_capacity(table + body_len);
    out.extend_from_slice(&(count as u32).to_le_bytes());
    let mut off = table as u32;
    for b in blocks {
        out.extend_from_slice(&off.to_le_bytes());
        off = off
            .checked_add(b.len() as u32)
            .ok_or("Opus chunk exceeds 4 GiB")?;
    }
    out.extend_from_slice(&off.to_le_bytes());
    for b in blocks {
        out.extend_from_slice(b);
    }
    Ok(out)
}

/// Splits a whole `ogop` file into `[head_chunk0, chunk1, ...]`, each holding
/// at most `per_chunk` Opus blocks. Chunk 0 carries the `ogop` header with
/// the whole sound's frame count; chunks `1..` use [`encode_chunk`].
pub fn split(bytes: &[u8], per_chunk: usize) -> Result<Vec<Vec<u8>>, String> {
    if per_chunk == 0 {
        return Err("Opus split: per_chunk must be at least 1".into());
    }
    let ix = index(bytes)?;
    let all: Vec<Vec<u8>> = ix
        .blocks
        .iter()
        .map(|&(a, b)| bytes[a..b].to_vec())
        .collect();
    split_blocks(
        &all,
        ix.channels,
        ix.frames,
        ix.source_rate,
        ix.bitrate,
        per_chunk,
    )
}

/// Assembles pre-encoded Opus blocks into `[head_chunk0, chunk1, ...]` of
/// `per_chunk` blocks each.
pub fn split_blocks(
    blocks: &[Vec<u8>],
    channels: usize,
    total_frames: u64,
    source_rate: u32,
    bitrate: u32,
    per_chunk: usize,
) -> Result<Vec<Vec<u8>>, String> {
    if per_chunk == 0 {
        return Err("Opus split: per_chunk must be at least 1".into());
    }
    let mut chunks = Vec::with_capacity(blocks.len().div_ceil(per_chunk));
    for (ci, slice) in blocks.chunks(per_chunk).enumerate() {
        if ci == 0 {
            chunks.push(encode_blocks_file(
                slice,
                channels,
                total_frames,
                source_rate,
                bitrate,
            )?);
        } else {
            chunks.push(encode_chunk(slice)?);
        }
    }
    Ok(chunks)
}

/// Streaming 48 kHz Opus block encoder: consumes interleaved 16-bit PCM
/// chunks with bounded memory (holding only the active block plus pre-roll
/// and lookahead) and emits self-contained Opus blocks identical to [`encode`].
pub struct StreamEncoder {
    channels: usize,
    bitrate: u32,
    lookahead: usize,
    /// Rolling PCM window starting at `window_start` sample frame.
    window: Vec<i16>,
    window_start: i64,
    total_frames: u64,
    blocks: Vec<Vec<u8>>,
}

impl StreamEncoder {
    pub fn new(channels: usize, bitrate: u32) -> Result<Self, String> {
        if !(1..=2).contains(&channels) {
            return Err(format!(
                "Opus sounds are mono or stereo, not {channels} channels"
            ));
        }
        let enc = OpusEncoder::new(RATE as i32, channels, Application::RestrictedLowDelay)
            .map_err(|e| format!("Opus encoder: {e}"))?;
        let lookahead = enc.lookahead();
        Ok(Self {
            channels,
            bitrate,
            lookahead,
            window: Vec::new(),
            window_start: 0,
            total_frames: 0,
            blocks: Vec::new(),
        })
    }

    /// Appends interleaved 48 kHz 16-bit PCM frames and encodes any blocks
    /// whose full input (including encoder lookahead) has arrived.
    pub fn push(&mut self, samples: &[i16]) -> Result<(), String> {
        if !samples.len().is_multiple_of(self.channels) {
            return Err("Opus stream encode: samples are not whole frames".into());
        }
        self.window.extend_from_slice(samples);
        self.total_frames += (samples.len() / self.channels) as u64;
        self.drain_ready(false)
    }

    /// Flushes remaining frames and returns `(encoded_blocks, total_frames)`.
    pub fn finish(mut self) -> Result<(Vec<Vec<u8>>, u64), String> {
        self.drain_ready(true)?;
        if self.blocks.is_empty() {
            // Silent 0-frame guard: encode one empty block as `encode` does.
            let b = encode_one_block(self.channels, self.bitrate, 0, 0, |_, _| 0)?;
            self.blocks.push(b);
        }
        Ok((self.blocks, self.total_frames))
    }

    fn drain_ready(&mut self, eof: bool) -> Result<(), String> {
        let ch = self.channels;
        let preroll = (PREROLL_PACKETS * PACKET_FRAMES) as i64;
        loop {
            let k = self.blocks.len();
            let block_start = (k as u64) * BLOCK_FRAMES;
            if eof {
                if block_start >= self.total_frames && k > 0 {
                    break;
                }
                if self.total_frames == 0 {
                    break;
                }
            } else {
                let need_end = block_start + BLOCK_FRAMES + self.lookahead as u64;
                if self.total_frames < need_end {
                    break;
                }
            }
            let frames = self.total_frames;
            let ws = self.window_start;
            let win = &self.window;
            let wframes = (win.len() / ch) as i64;
            let body = encode_one_block(ch, self.bitrate, k, frames, |i, c| {
                if i < 0 || i as u64 >= frames {
                    0
                } else {
                    let rel = i - ws;
                    if rel < 0 || rel >= wframes {
                        0
                    } else {
                        win[rel as usize * ch + c]
                    }
                }
            })?;
            self.blocks.push(body);
            // Keep at least `preroll` frames before the next block's start.
            let next_start = ((k + 1) as u64 * BLOCK_FRAMES) as i64;
            let keep_from = (next_start - preroll).max(0);
            if keep_from > self.window_start {
                let drop_frames = ((keep_from - self.window_start) as usize).min(win.len() / ch);
                self.window.drain(..drop_frames * ch);
                self.window_start += drop_frames as i64;
            }
            if eof && block_start + BLOCK_FRAMES >= self.total_frames {
                break;
            }
        }
        Ok(())
    }
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
    let at = |i: i64, c: usize| -> i16 {
        if i < 0 || i as u64 >= frames {
            0
        } else {
            samples[i as usize * channels + c]
        }
    };
    let mut encoded = Vec::with_capacity(blocks);
    for k in 0..blocks {
        encoded.push(encode_one_block(channels, bitrate, k, frames, at)?);
    }
    encode_blocks_file(&encoded, channels, frames, source_rate, bitrate)
}

/// Whether `bytes` start like an Opus sound.
pub fn is_opus(bytes: &[u8]) -> bool {
    bytes.get(0..4) == Some(&MAGIC.to_le_bytes()[..])
}

fn index_inner(bytes: &[u8], whole: bool) -> Result<Index, String> {
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
    let total_blocks = block_count(frames);
    if whole {
        if count != total_blocks {
            return Err(bad(&format!("{count} blocks for {frames} samples")));
        }
    } else if count == 0 || count > total_blocks {
        return Err(bad(&format!(
            "{count} head blocks out of {total_blocks} for {frames} samples"
        )));
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

/// Reads the header and block table of a whole Opus sound, checking them
/// against the file.
pub fn index(bytes: &[u8]) -> Result<Index, String> {
    index_inner(bytes, true)
}

/// Reads the header and block table of a chunked Opus sound's head (chunk 0),
/// whose `frames` declares the full track length and whose `blocks` holds
/// chunk 0's blocks only.
pub fn index_head(bytes: &[u8]) -> Result<Index, String> {
    index_inner(bytes, false)
}

/// Indexes an Opus chunk (`n >= 1`, written by [`encode_chunk`]): validates
/// that it holds `expected_blocks` blocks and returns their byte ranges.
pub fn index_chunk(bytes: &[u8], expected_blocks: usize) -> Result<Vec<(usize, usize)>, String> {
    let u32_at = |o: usize| -> Option<u32> {
        Some(u32::from_le_bytes(bytes.get(o..o + 4)?.try_into().ok()?))
    };
    let bad = |what: &str| format!("malformed Opus chunk: {what}");
    let count = u32_at(0).ok_or_else(|| bad("no block count"))? as usize;
    if count != expected_blocks || count == 0 {
        return Err(bad(&format!(
            "has {count} blocks, expected {expected_blocks}"
        )));
    }
    let table = 4 + 4 * (count + 1);
    let mut blocks = Vec::with_capacity(count);
    for k in 0..count {
        let a = u32_at(4 + 4 * k).ok_or_else(|| bad("truncated block table"))? as usize;
        let b = u32_at(4 + 4 * (k + 1)).ok_or_else(|| bad("truncated block table"))? as usize;
        if a < table || b < a || b > bytes.len() {
            return Err(bad(&format!("block {k} lies outside the chunk")));
        }
        blocks.push((a, b));
    }
    if blocks.last().is_some_and(|&(_, end)| end != bytes.len()) {
        return Err(bad("bytes past the last block"));
    }
    Ok(blocks)
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
