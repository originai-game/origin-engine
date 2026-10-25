//! How an animation blob's channels hold their samples (`format::ChannelRow`,
//! [`codec`]): f32 as resampled, one sample for a constant channel, or
//! quantised keys with keyframe reduction (an `animation { compress true }`
//! import setting, within its `tolerance`).
//!
//! - **Rotations** are 48-bit smallest-three quaternions: the index of the
//!   largest component (made positive) in 2 bits and the other three in
//!   15 bits each over `[-1/√2, 1/√2]`; the largest is rebuilt from the
//!   unit length.
//! - **Translations, scales and morph weights** are 16-bit unorm per lane
//!   over the channel's own range (`min`, `extent` per lane, in `samples`).
//! - **Keyframe reduction** keeps the first and last frame and drops every
//!   frame that the interpolation between its kept neighbours (lerp, or
//!   normalised lerp on the shorter arc for rotations; a hold for STEP
//!   channels) rebuilds within the tolerance of the resampled value. The
//!   loader keeps the keys as stored ([`samples`], `og_anim::Samples`) and
//!   og-anim rebuilds a dropped frame the same way when it samples it
//!   (the decoding arithmetic is og-anim's, which this encoder measures
//!   against), so a resident clip costs what its cooked form costs and
//!   poses exactly as the same frames written out as f32 would.
//!
//! - **Visibility** (`path::VISIBILITY`, 0 or 1, STEP) is always kept
//!   exact: its tolerance is 0 whatever the setting, so the reduction
//!   keeps exactly the frames where the flag changes (and the ends), and
//!   a unorm over `[0, 1]` rebuilds 0 and 1 bit for bit.
//!
//! A channel whose quantisation alone misses the tolerance (a range too
//! wide for 16 bits) stays f32. A channel whose frames are all the same
//! bits is always stored once (lossless). Decoding uses only `+ - * /` and
//! `sqrt` in a fixed order, so every host rebuilds the same floats.

use crate::format::{ChannelRow, path};
use og_anim::{QuantizedKeys, Samples, dequantize, nlerp4, normalize4, unpack_quat};

/// [`ChannelRow::codec`] values.
pub mod codec {
    /// `frames × lanes` f32 at `sample_offset`.
    pub const F32: u32 = 0;
    /// One sample (`lanes` f32 at `sample_offset`) for every frame.
    pub const CONSTANT: u32 = 1;
    /// `keys` quantised samples at `packed_offset` (u16s of `packed`): when
    /// `keys < frames`, first the kept frames' indices (ascending, the first
    /// 0 and the last `frames - 1`), then per key 3 u16 (a rotation's
    /// smallest-three) or `lanes` u16 (unorm over `min + extent · q / 65535`,
    /// `min` and `extent` per lane at `sample_offset`).
    pub const QUANTIZED: u32 = 2;
}

/// The largest frame count a quantised channel indexes (its key indices are u16).
pub const MAX_QUANTIZED_FRAMES: u32 = 65535;

const SQRT1_2: f32 = std::f32::consts::FRAC_1_SQRT_2;
const Q15: f32 = 32767.0;

/// A unit quaternion as smallest-three: three u16 words (bits 47..0:
/// the largest component's index in 46..45, then the others in order,
/// 15 bits each).
fn pack_quat(q: [f32; 4]) -> [u16; 3] {
    let q = normalize4(q);
    let mut big = 0;
    for i in 1..4 {
        if q[i].abs() > q[big].abs() {
            big = i;
        }
    }
    let sign = if q[big] < 0.0 { -1.0 } else { 1.0 };
    let mut bits: u64 = (big as u64) << 45;
    let mut shift = 30;
    for (i, &c) in q.iter().enumerate() {
        if i == big {
            continue;
        }
        let x = (c * sign / SQRT1_2).clamp(-1.0, 1.0);
        let v = ((x * 0.5 + 0.5) * Q15).round() as u64;
        bits |= v << shift;
        shift -= 15;
    }
    [(bits >> 32) as u16, (bits >> 16) as u16, bits as u16]
}

/// A channel's frames as og-anim keeps them resident, from its row and the
/// blob's `samples` and `packed` sections: f32 and constant samples
/// copied, quantised keys kept as stored (og-anim samples them where they
/// lie, `og_anim::Samples`).
pub fn samples(
    row: &ChannelRow,
    frames: u32,
    samples: &[f32],
    packed: &[u16],
) -> Result<Samples, String> {
    let lanes = row.lanes as usize;
    let n = frames as usize;
    let at = row.sample_offset as usize;
    let floats = |len: usize| {
        samples
            .get(at..at + len)
            .ok_or_else(|| "channel samples out of range".to_string())
    };
    match row.codec {
        codec::F32 => Ok(Samples::F32(floats(n * lanes)?.to_vec())),
        codec::CONSTANT => Ok(Samples::Constant(floats(lanes)?.to_vec())),
        codec::QUANTIZED => {
            let rotation = row.path == path::ROTATION;
            let keys = row.keys as usize;
            if keys == 0 || keys > n || (rotation && lanes != 4) {
                return Err(format!("quantised channel with {keys} keys of {n} frames"));
            }
            let words = if rotation { 3 } else { lanes };
            let index_len = if keys < n { keys } else { 0 };
            let start = row.packed_offset as usize;
            let data = packed
                .get(start..start + index_len + keys * words)
                .ok_or_else(|| "quantised channel out of range".to_string())?;
            let (index, values) = data.split_at(index_len);
            if index_len > 0
                && (index[0] != 0
                    || usize::from(index[keys - 1]) != n - 1
                    || index.windows(2).any(|w| w[1] <= w[0]))
            {
                return Err(
                    "quantised channel key frames are not ascending from 0 to the last".into(),
                );
            }
            Ok(Samples::Quantized(QuantizedKeys {
                frames: index.to_vec(),
                words: values.to_vec(),
                range: if rotation {
                    Vec::new()
                } else {
                    floats(2 * lanes)?.to_vec()
                },
            }))
        }
        c => Err(format!("channel codec {c}")),
    }
}

/// The f32 samples of a channel, `frames × lanes`: every frame of
/// [`samples`] written out (what og-anim samples, frame for frame).
pub fn decode(
    row: &ChannelRow,
    frames: u32,
    samples_: &[f32],
    packed: &[u16],
) -> Result<Vec<f32>, String> {
    let channel = og_anim::Channel {
        node: row.node,
        node_name: String::new(),
        path: og_anim::ChannelPath::from_code(row.path)
            .ok_or_else(|| format!("channel path {}", row.path))?,
        step: row_step(row),
        lanes: row.lanes,
        samples: samples(row, frames, samples_, packed)?,
    };
    Ok(channel.decoded(frames))
}

fn row_step(row: &ChannelRow) -> bool {
    row.interpolation == crate::format::interpolation::STEP
}

/// The value at frame `j` between kept frames `fa` (value `a`) and `fb`.
fn between(
    a: &[f32],
    b: &[f32],
    rotation: bool,
    step: bool,
    fa: usize,
    fb: usize,
    j: usize,
) -> Vec<f32> {
    if step {
        return a.to_vec();
    }
    let u = (j - fa) as f32 / (fb - fa) as f32;
    if rotation {
        nlerp4([a[0], a[1], a[2], a[3]], [b[0], b[1], b[2], b[3]], u).to_vec()
    } else {
        a.iter().zip(b).map(|(&x, &y)| x + (y - x) * u).collect()
    }
}

/// A channel as the cook stores it: the row's codec fields and its floats
/// and u16s (offsets are the writer's).
#[derive(Clone, Debug, PartialEq)]
pub struct Encoded {
    pub codec: u32,
    pub keys: u32,
    /// Appended to `samples` at the row's `sample_offset`.
    pub floats: Vec<f32>,
    /// Appended to `packed` at the row's `packed_offset`.
    pub words: Vec<u16>,
}

/// The largest per-lane difference between `a` and `b` (a rotation's sign
/// ignored: q and -q are one rotation).
fn error(a: &[f32], b: &[f32], rotation: bool) -> f32 {
    let flip = rotation && a.iter().zip(b).map(|(x, y)| x * y).sum::<f32>() < 0.0;
    a.iter()
        .zip(b)
        .map(|(&x, &y)| (x - if flip { -y } else { y }).abs())
        .fold(0.0, f32::max)
}

/// Encodes one channel's `frames × lanes` samples. `tolerance` None keeps
/// f32 (a constant channel is still stored once, losslessly).
pub fn encode(
    path_code: u32,
    step: bool,
    lanes: u32,
    frames: u32,
    data: &[f32],
    tolerance: Option<f32>,
) -> Encoded {
    let lanes_n = lanes as usize;
    let n = frames as usize;
    assert_eq!(data.len(), n * lanes_n);
    let frame = |j: usize| &data[j * lanes_n..(j + 1) * lanes_n];
    let f32s = || Encoded {
        codec: codec::F32,
        keys: frames,
        floats: data.to_vec(),
        words: Vec::new(),
    };
    let constant = |first: &[f32]| Encoded {
        codec: codec::CONSTANT,
        keys: 1,
        floats: first.to_vec(),
        words: Vec::new(),
    };
    if n > 1
        && (1..n).all(|j| {
            frame(j)
                .iter()
                .zip(frame(0))
                .all(|(a, b)| a.to_bits() == b.to_bits())
        })
    {
        return constant(frame(0));
    }
    let Some(tol) = tolerance else {
        return f32s();
    };
    // A flag is never approximated.
    let tol = if path_code == path::VISIBILITY {
        0.0
    } else {
        tol
    };
    let rotation = path_code == path::ROTATION;
    if n <= 1 || frames > MAX_QUANTIZED_FRAMES || (rotation && lanes != 4) || lanes == 0 {
        return f32s();
    }
    if (1..n).all(|j| error(frame(j), frame(0), rotation) <= tol) {
        return constant(frame(0));
    }
    // Quantise every frame, and check the quantisation alone.
    let mut range = Vec::new();
    let mut quantized: Vec<Vec<u16>> = Vec::with_capacity(n);
    let mut deq: Vec<Vec<f32>> = Vec::with_capacity(n);
    if rotation {
        for j in 0..n {
            let f = frame(j);
            let w = pack_quat([f[0], f[1], f[2], f[3]]);
            deq.push(unpack_quat(w).to_vec());
            quantized.push(w.to_vec());
        }
    } else {
        for l in 0..lanes_n {
            let (lo, hi) = (0..n)
                .map(|j| frame(j)[l])
                .fold((f32::MAX, f32::MIN), |(a, b), x| (a.min(x), b.max(x)));
            range.push(lo);
            range.push(hi - lo);
        }
        for j in 0..n {
            let f = frame(j);
            let q: Vec<u16> = (0..lanes_n)
                .map(|l| {
                    let (lo, ext) = (range[2 * l], range[2 * l + 1]);
                    if ext > 0.0 {
                        (((f[l] - lo) / ext).clamp(0.0, 1.0) * 65535.0).round() as u16
                    } else {
                        0
                    }
                })
                .collect();
            deq.push(
                (0..lanes_n)
                    .map(|l| dequantize(range[2 * l], range[2 * l + 1], q[l]))
                    .collect(),
            );
            quantized.push(q);
        }
    }
    if range.iter().any(|x| !x.is_finite())
        || (0..n).any(|j| error(&deq[j], frame(j), rotation) > tol)
    {
        return f32s();
    }
    // Keyframe reduction: from each kept frame, the farthest next frame
    // whose interpolation rebuilds every frame between within `tol`.
    const MAX_SPAN: usize = 1024;
    let mut kept = vec![0usize];
    let mut a = 0;
    while a < n - 1 {
        let mut b = a + 1;
        for cand in a + 2..n.min(a + MAX_SPAN + 1) {
            let fits = (a + 1..cand).all(|j| {
                let v = between(&deq[a], &deq[cand], rotation, step, a, cand, j);
                error(&v, frame(j), rotation) <= tol
            });
            if !fits {
                break;
            }
            b = cand;
        }
        kept.push(b);
        a = b;
    }
    let mut words: Vec<u16> = Vec::new();
    if kept.len() < n {
        words.extend(kept.iter().map(|&k| k as u16));
    }
    for &k in &kept {
        words.extend_from_slice(&quantized[k]);
    }
    Encoded {
        codec: codec::QUANTIZED,
        keys: kept.len() as u32,
        floats: range,
        words,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::interpolation;

    fn row(e: &Encoded, path: u32, lanes: u32, step: bool) -> ChannelRow {
        ChannelRow {
            node: 0,
            path,
            interpolation: if step {
                interpolation::STEP
            } else {
                interpolation::LINEAR
            },
            lanes,
            sample_offset: 0,
            codec: e.codec,
            keys: e.keys,
            packed_offset: 0,
        }
    }

    fn round_trip(path: u32, lanes: u32, data: &[f32], tol: Option<f32>) -> (Encoded, Vec<f32>) {
        let frames = data.len() as u32 / lanes;
        let e = encode(path, false, lanes, frames, data, tol);
        let out = decode(&row(&e, path, lanes, false), frames, &e.floats, &e.words).unwrap();
        (e, out)
    }

    #[test]
    fn smallest_three_is_within_its_step() {
        let mut worst = 0.0f32;
        for i in 0..2000 {
            let t = i as f32 * 0.0137;
            let q = normalize4([
                t.sin(),
                (t * 1.7).cos(),
                (t * 0.3).sin() * 0.5,
                (t * 2.9).cos(),
            ]);
            let d = unpack_quat(pack_quat(q));
            worst = worst.max(error(&q, &d, true));
        }
        assert!(worst < 6e-5, "{worst}");
    }

    #[test]
    fn constant_channels_are_one_sample_and_exact() {
        let data: Vec<f32> = (0..30).flat_map(|_| [1.0, 0.25, -3.5]).collect();
        let (e, out) = round_trip(path::SCALE, 3, &data, None);
        assert_eq!(e.codec, codec::CONSTANT);
        assert_eq!(e.floats.len(), 3);
        assert_eq!(out, data);
    }

    #[test]
    fn without_a_tolerance_samples_stay_f32() {
        let data: Vec<f32> = (0..30).flat_map(|i| [i as f32 * 0.1, 0.0, 1.0]).collect();
        let (e, out) = round_trip(path::TRANSLATION, 3, &data, None);
        assert_eq!(e.codec, codec::F32);
        assert_eq!(out, data);
    }

    #[test]
    fn a_straight_line_keeps_two_keys_and_a_curve_stays_within_tolerance() {
        let line: Vec<f32> = (0..61)
            .flat_map(|i| [i as f32 * 0.05, 1.0 - i as f32 * 0.01, 2.0])
            .collect();
        let (e, out) = round_trip(path::TRANSLATION, 3, &line, Some(1e-4));
        assert_eq!((e.codec, e.keys), (codec::QUANTIZED, 2));
        assert!(out.iter().zip(&line).all(|(a, b)| (a - b).abs() <= 1e-4));
        let curve: Vec<f32> = (0..241)
            .flat_map(|i| {
                let t = i as f32 / 240.0 * std::f32::consts::TAU;
                [t.sin(), (2.0 * t).cos() * 0.3, t * 0.1]
            })
            .collect();
        let (e, out) = round_trip(path::TRANSLATION, 3, &curve, Some(1e-3));
        assert!(e.keys > 2 && e.keys < 241, "{}", e.keys);
        assert!(out.iter().zip(&curve).all(|(a, b)| (a - b).abs() <= 1e-3));
    }

    #[test]
    fn rotations_rebuild_within_tolerance() {
        let rot: Vec<f32> = (0..300)
            .flat_map(|i| {
                let a = i as f32 * 0.01;
                let (s, c) = ((a * 0.5).sin(), (a * 0.5).cos());
                let (s2, c2) = ((a * 0.2).sin(), (a * 0.2).cos());
                // A turn about y, then about x.
                normalize4([c * s2, s * c2, -s * s2, c * c2])
            })
            .collect();
        let (e, out) = round_trip(path::ROTATION, 4, &rot, Some(1e-4));
        assert_eq!(e.codec, codec::QUANTIZED);
        assert!(e.keys < 300, "{}", e.keys);
        for (a, b) in out.chunks(4).zip(rot.chunks(4)) {
            assert!(error(a, b, true) <= 1e-4);
        }
    }

    #[test]
    fn a_visibility_channel_keeps_its_changes_exactly_at_any_tolerance() {
        let data: Vec<f32> = [1, 1, 1, 0, 0, 0, 0, 1, 0, 0]
            .iter()
            .map(|&v| v as f32)
            .collect();
        let frames = data.len() as u32;
        let e = encode(path::VISIBILITY, true, 1, frames, &data, Some(0.1));
        let out = decode(
            &row(&e, path::VISIBILITY, 1, true),
            frames,
            &e.floats,
            &e.words,
        )
        .unwrap();
        assert_eq!(out, data);
        // The ends and the frames where it changes: 0, 3, 7, 8, 9.
        assert_eq!((e.codec, e.keys), (codec::QUANTIZED, 5));
        assert_eq!(&e.words[..5], &[0, 3, 7, 8, 9]);
    }

    #[test]
    fn a_range_too_wide_for_sixteen_bits_stays_f32() {
        let data: Vec<f32> = (0..10)
            .flat_map(|i| [i as f32 * 1000.0, 0.0, 0.0])
            .collect();
        let (e, out) = round_trip(path::TRANSLATION, 3, &data, Some(1e-4));
        assert_eq!(e.codec, codec::F32);
        assert_eq!(out, data);
    }
}
