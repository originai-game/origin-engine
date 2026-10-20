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
//!   loader rebuilds the dropped frames the same way, so a clip has every
//!   frame again and og-anim samples it exactly as an f32 clip.
//!
//! A channel whose quantisation alone misses the tolerance (a range too
//! wide for 16 bits) stays f32. A channel whose frames are all the same
//! bits is always stored once (lossless). Decoding uses only `+ - * /` and
//! `sqrt` in a fixed order, so every host rebuilds the same floats.

use crate::format::{ChannelRow, path};

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

fn normalize4(q: [f32; 4]) -> [f32; 4] {
    let len2 = q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3];
    if len2 > 1e-24 && len2.is_finite() {
        let s = 1.0 / len2.sqrt();
        [q[0] * s, q[1] * s, q[2] * s, q[3] * s]
    } else {
        [0.0, 0.0, 0.0, 1.0]
    }
}

fn dot4(a: [f32; 4], b: [f32; 4]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]
}

/// Normalised lerp on the shorter arc, scalar and in a fixed order.
fn nlerp4(a: [f32; 4], b: [f32; 4], u: f32) -> [f32; 4] {
    let b = if dot4(a, b) < 0.0 {
        [-b[0], -b[1], -b[2], -b[3]]
    } else {
        b
    };
    let v = 1.0 - u;
    normalize4([
        a[0] * v + b[0] * u,
        a[1] * v + b[1] * u,
        a[2] * v + b[2] * u,
        a[3] * v + b[3] * u,
    ])
}

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

fn unpack_quat(w: [u16; 3]) -> [f32; 4] {
    let bits = (u64::from(w[0]) << 32) | (u64::from(w[1]) << 16) | u64::from(w[2]);
    let big = ((bits >> 45) & 3) as usize;
    let mut q = [0.0f32; 4];
    let mut shift = 30;
    let mut sum = 0.0f32;
    for (i, c) in q.iter_mut().enumerate() {
        if i == big {
            continue;
        }
        let v = ((bits >> shift) & 0x7fff) as f32;
        let x = (v / Q15 * 2.0 - 1.0) * SQRT1_2;
        *c = x;
        sum += x * x;
        shift -= 15;
    }
    q[big] = (1.0 - sum).max(0.0).sqrt();
    normalize4(q)
}

fn dequantize(min: f32, extent: f32, q: u16) -> f32 {
    min + extent * (f32::from(q) / 65535.0)
}

/// The f32 samples of a channel, `frames × lanes`, from its row and the
/// blob's `samples` and `packed` sections.
pub fn decode(
    row: &ChannelRow,
    frames: u32,
    samples: &[f32],
    packed: &[u16],
) -> Result<Vec<f32>, String> {
    let lanes = row.lanes as usize;
    let n = frames as usize;
    let at = row.sample_offset as usize;
    let floats = |len: usize| {
        samples
            .get(at..at + len)
            .ok_or_else(|| "channel samples out of range".to_string())
    };
    match row.codec {
        codec::F32 => Ok(floats(n * lanes)?.to_vec()),
        codec::CONSTANT => {
            let one = floats(lanes)?;
            Ok((0..n).flat_map(|_| one.iter().copied()).collect())
        }
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
            let range = if rotation {
                &[][..]
            } else {
                floats(2 * lanes)?
            };
            let key_value = |k: usize| -> Vec<f32> {
                let v = &values[k * words..(k + 1) * words];
                if rotation {
                    unpack_quat([v[0], v[1], v[2]]).to_vec()
                } else {
                    (0..lanes)
                        .map(|l| dequantize(range[2 * l], range[2 * l + 1], v[l]))
                        .collect()
                }
            };
            let frame_of = |k: usize| if index_len > 0 { index[k] as usize } else { k };
            if index_len > 0
                && (frame_of(0) != 0
                    || frame_of(keys - 1) != n - 1
                    || index.windows(2).any(|w| w[1] <= w[0]))
            {
                return Err(
                    "quantised channel key frames are not ascending from 0 to the last".into(),
                );
            }
            let mut out = Vec::with_capacity(n * lanes);
            let mut a = key_value(0);
            out.extend_from_slice(&a);
            for k in 1..keys {
                let b = key_value(k);
                let (fa, fb) = (frame_of(k - 1), frame_of(k));
                for j in fa + 1..fb {
                    out.extend(between(&a, &b, rotation, row_step(row), fa, fb, j));
                }
                out.extend_from_slice(&b);
                a = b;
            }
            Ok(out)
        }
        c => Err(format!("channel codec {c}")),
    }
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
    fn a_range_too_wide_for_sixteen_bits_stays_f32() {
        let data: Vec<f32> = (0..10)
            .flat_map(|i| [i as f32 * 1000.0, 0.0, 0.0])
            .collect();
        let (e, out) = round_trip(path::TRANSLATION, 3, &data, Some(1e-4));
        assert_eq!(e.codec, codec::F32);
        assert_eq!(out, data);
    }
}
