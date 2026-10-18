//! QOA, the "Quite OK Audio" format (<https://qoaformat.org>, reference
//! `qoa.h` by Dominic Szablewski, MIT): 16-bit PCM in 3.2 bits per sample
//! with a 4-tap LMS predictor. A small encoder for the cook and a decoder
//! for the runtime, both pure Rust and deterministic (integer arithmetic
//! only), so the same WAV cooks to the same bytes everywhere.
//!
//! File: `qoaf` (u32 BE), samples per channel (u32 BE), then frames of up to
//! 5120 samples per channel: an 8-byte frame header (channels u8, sample rate
//! u24, samples u16, frame bytes u16), per channel the LMS history and
//! weights (4 x i16 each, BE), then 20-sample slices interleaved by channel
//! (u64 BE: 4-bit scale factor, 20 x 3-bit residuals).

const SLICE_LEN: usize = 20;
const SLICES_PER_FRAME: usize = 256;
const FRAME_LEN: usize = SLICE_LEN * SLICES_PER_FRAME;
const LMS_LEN: usize = 4;
const MAGIC: u32 = 0x716f_6166; // "qoaf"
/// The most channels a QOA frame header can name.
pub const MAX_CHANNELS: usize = 8;

const QUANT_TAB: [i32; 17] = [7, 7, 7, 5, 5, 3, 3, 1, 0, 0, 2, 2, 4, 4, 6, 6, 6];
const RECIPROCAL_TAB: [i32; 16] = [
    65536, 9363, 3121, 1457, 781, 475, 311, 216, 156, 117, 90, 71, 57, 47, 39, 32,
];
const DEQUANT_TAB: [[i32; 8]; 16] = [
    [1, -1, 3, -3, 5, -5, 7, -7],
    [5, -5, 18, -18, 32, -32, 49, -49],
    [16, -16, 53, -53, 95, -95, 147, -147],
    [34, -34, 113, -113, 203, -203, 315, -315],
    [63, -63, 210, -210, 378, -378, 588, -588],
    [104, -104, 345, -345, 621, -621, 966, -966],
    [158, -158, 528, -528, 950, -950, 1477, -1477],
    [228, -228, 760, -760, 1368, -1368, 2128, -2128],
    [316, -316, 1053, -1053, 1895, -1895, 2947, -2947],
    [422, -422, 1405, -1405, 2529, -2529, 3934, -3934],
    [548, -548, 1828, -1828, 3290, -3290, 5117, -5117],
    [696, -696, 2320, -2320, 4176, -4176, 6496, -6496],
    [868, -868, 2893, -2893, 5207, -5207, 8099, -8099],
    [1064, -1064, 3548, -3548, 6386, -6386, 9933, -9933],
    [1286, -1286, 4288, -4288, 7718, -7718, 12005, -12005],
    [1536, -1536, 5120, -5120, 9216, -9216, 14336, -14336],
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Lms {
    history: [i32; LMS_LEN],
    weights: [i32; LMS_LEN],
}

impl Lms {
    fn new() -> Lms {
        Lms {
            history: [0; LMS_LEN],
            weights: [0, 0, -(1 << 13), 1 << 14],
        }
    }

    fn predict(&self) -> i32 {
        let mut p: i32 = 0;
        for i in 0..LMS_LEN {
            p = p.wrapping_add(self.weights[i].wrapping_mul(self.history[i]));
        }
        p >> 13
    }

    fn update(&mut self, sample: i32, residual: i32) {
        let delta = residual >> 4;
        for i in 0..LMS_LEN {
            self.weights[i] += if self.history[i] < 0 { -delta } else { delta };
        }
        self.history.copy_within(1.., 0);
        self.history[LMS_LEN - 1] = sample;
    }
}

fn clamp_s16(v: i32) -> i32 {
    v.clamp(-32768, 32767)
}

fn div(v: i32, scalefactor: usize) -> i32 {
    let reciprocal = RECIPROCAL_TAB[scalefactor];
    // In i64: a full-scale residual times the largest reciprocal overflows i32.
    let n = ((i64::from(v) * i64::from(reciprocal) + (1 << 15)) >> 16) as i32;
    n + ((v > 0) as i32 - (v < 0) as i32) - ((n > 0) as i32 - (n < 0) as i32)
}

/// Encodes interleaved 16-bit samples (`channels` per frame, 1..=8).
pub fn encode(samples: &[i16], channels: usize, sample_rate: u32) -> Result<Vec<u8>, String> {
    if channels == 0 || channels > MAX_CHANNELS {
        return Err(format!(
            "{channels} channels; QOA carries 1 to {MAX_CHANNELS}"
        ));
    }
    if sample_rate == 0 || sample_rate > 0xff_ffff {
        return Err(format!(
            "sample rate {sample_rate} does not fit QOA's 24 bits"
        ));
    }
    if !samples.len().is_multiple_of(channels) {
        return Err("sample count is not a multiple of the channel count".into());
    }
    let frames = samples.len() / channels;
    if frames > u32::MAX as usize {
        return Err("too many samples for one QOA file".into());
    }
    let mut out = Vec::with_capacity(8 + frames * channels / 2 + 64);
    out.extend_from_slice(&MAGIC.to_be_bytes());
    out.extend_from_slice(&(frames as u32).to_be_bytes());
    let mut lms = vec![Lms::new(); channels];
    let mut prev_sf = vec![0usize; channels];
    let mut start = 0;
    while start < frames {
        let len = FRAME_LEN.min(frames - start);
        let slices = len.div_ceil(SLICE_LEN);
        let frame_size = 8 + LMS_LEN * 4 * channels + 8 * slices * channels;
        out.push(channels as u8);
        out.extend_from_slice(&sample_rate.to_be_bytes()[1..]);
        out.extend_from_slice(&(len as u16).to_be_bytes());
        out.extend_from_slice(&(frame_size as u16).to_be_bytes());
        for l in &lms {
            let mut h: u64 = 0;
            let mut w: u64 = 0;
            for i in 0..LMS_LEN {
                h = (h << 16) | (l.history[i] as i16 as u16 as u64);
                w = (w << 16) | (l.weights[i] as i16 as u16 as u64);
            }
            out.extend_from_slice(&h.to_be_bytes());
            out.extend_from_slice(&w.to_be_bytes());
        }
        for slice_start in (0..len).step_by(SLICE_LEN) {
            let slice_len = SLICE_LEN.min(len - slice_start);
            for c in 0..channels {
                let at = |i: usize| samples[(start + slice_start + i) * channels + c] as i32;
                // Try every scale factor (starting from the previous one, as
                // the reference does) and keep the one with the least error.
                let mut best: Option<(u64, u64, usize, Lms)> = None;
                for sfi in 0..16 {
                    let sf = (sfi + prev_sf[c]) % 16;
                    let mut l = lms[c];
                    let mut slice: u64 = sf as u64;
                    let mut error: u64 = 0;
                    for i in 0..slice_len {
                        let sample = at(i);
                        let predicted = l.predict();
                        let residual = sample - predicted;
                        let scaled = div(residual, sf).clamp(-8, 8);
                        let quantized = QUANT_TAB[(scaled + 8) as usize];
                        let dequantized = DEQUANT_TAB[sf][quantized as usize];
                        let reconstructed = clamp_s16(predicted + dequantized);
                        let weights_penalty =
                            (l.weights.iter().map(|&w| w as i64 * w as i64).sum::<i64>() >> 18)
                                - 0x8ff;
                        let mut e = (sample - reconstructed) as i64;
                        e *= e;
                        if weights_penalty > 0 {
                            e += weights_penalty * weights_penalty;
                        }
                        error = error.saturating_add(e as u64);
                        if best.is_some_and(|(be, ..)| error >= be) {
                            break;
                        }
                        l.update(reconstructed, dequantized);
                        slice = (slice << 3) | quantized as u64;
                    }
                    if best.is_none_or(|(be, ..)| error < be) {
                        best = Some((error, slice, sf, l));
                    }
                }
                let (_, mut slice, sf, l) = best.expect("sixteen candidates");
                prev_sf[c] = sf;
                lms[c] = l;
                slice <<= (SLICE_LEN - slice_len) * 3;
                out.extend_from_slice(&slice.to_be_bytes());
            }
        }
        start += len;
    }
    Ok(out)
}

/// A decoded QOA file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decoded {
    pub channels: usize,
    pub sample_rate: u32,
    /// Interleaved samples, `frames * channels` long.
    pub samples: Vec<i16>,
}

fn be_u64(b: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_be_bytes(b.get(at..at + 8)?.try_into().ok()?))
}

/// Decodes a QOA file.
pub fn decode(bytes: &[u8]) -> Result<Decoded, String> {
    let bad = |what: &str| format!("malformed QOA: {what}");
    let header = be_u64(bytes, 0).ok_or_else(|| bad("no header"))?;
    if (header >> 32) as u32 != MAGIC {
        return Err(bad("wrong magic"));
    }
    let frames = (header & 0xffff_ffff) as usize;
    let mut pos = 8;
    let mut channels = 0usize;
    let mut sample_rate = 0u32;
    let mut samples: Vec<i16> = Vec::new();
    let mut decoded = 0usize;
    while decoded < frames {
        let fh = be_u64(bytes, pos).ok_or_else(|| bad("truncated frame header"))?;
        let ch = (fh >> 56) as usize;
        let rate = ((fh >> 32) & 0xff_ffff) as u32;
        let len = ((fh >> 16) & 0xffff) as usize;
        let size = (fh & 0xffff) as usize;
        if ch == 0 || ch > MAX_CHANNELS || len == 0 || len > FRAME_LEN {
            return Err(bad("invalid frame header"));
        }
        if channels == 0 {
            channels = ch;
            sample_rate = rate;
            samples.reserve(frames * ch);
        } else if ch != channels || rate != sample_rate {
            return Err(bad("channel count or sample rate changes between frames"));
        }
        let slices = len.div_ceil(SLICE_LEN);
        if size != 8 + LMS_LEN * 4 * ch + 8 * slices * ch || pos + size > bytes.len() {
            return Err(bad("frame size does not match its header"));
        }
        pos += 8;
        let mut lms = vec![Lms::new(); ch];
        for l in lms.iter_mut() {
            let mut h = be_u64(bytes, pos).expect("bounded by the frame size");
            let mut w = be_u64(bytes, pos + 8).expect("bounded by the frame size");
            for i in 0..LMS_LEN {
                l.history[i] = (h >> 48) as i16 as i32;
                h <<= 16;
                l.weights[i] = (w >> 48) as i16 as i32;
                w <<= 16;
            }
            pos += 16;
        }
        let base = samples.len();
        samples.resize(base + len * ch, 0);
        for slice_start in (0..len).step_by(SLICE_LEN) {
            for (c, l) in lms.iter_mut().enumerate() {
                let mut slice = be_u64(bytes, pos).expect("bounded by the frame size");
                pos += 8;
                let sf = (slice >> 60) as usize;
                slice <<= 4;
                for i in slice_start..(slice_start + SLICE_LEN).min(len) {
                    let quantized = (slice >> 61) as usize;
                    slice <<= 3;
                    let predicted = l.predict();
                    let dequantized = DEQUANT_TAB[sf][quantized];
                    let reconstructed = clamp_s16(predicted + dequantized);
                    samples[base + i * ch + c] = reconstructed as i16;
                    l.update(reconstructed, dequantized);
                }
            }
        }
        decoded += len;
    }
    if decoded != frames {
        return Err(bad("frames do not add up to the header's sample count"));
    }
    Ok(Decoded {
        channels: channels.max(1),
        sample_rate,
        samples,
    })
}
