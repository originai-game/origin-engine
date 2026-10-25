//! How a clip channel holds its frames in memory ([`Samples`]): f32 as
//! resampled, one sample for a constant channel, or the quantised keys the
//! cook stored (og-cook's `anim_codec`, an `animation { compress true }`
//! import setting), sampled where they lie, so a resident clip costs what
//! its cooked form costs.
//!
//! A quantised channel keeps its kept frames' indices (none when every
//! frame was kept), three u16 per rotation key (smallest-three: the index
//! of the largest component in 2 bits, the others in 15 bits each over
//! `[-1/√2, 1/√2]`) or one u16 per lane of any other key (unorm over the
//! lane's `min + extent · q / 65535`), and the lanes' ranges. Frame `j`
//! is its key when it was kept, else the interpolation between the kept
//! frames around it (`fa < j < fb`, `u = (j − fa) / (fb − fa)`): a hold for
//! STEP channels, a lerp per lane, a normalised lerp on the shorter arc for
//! rotations. That is the arithmetic the cook's keyframe reduction measured
//! its tolerance against, in the same order (`+ - * /` and `sqrt` only),
//! so a frame sampled here has the bits a decoder writing every frame out
//! would have given it, on every host, and a pose sampled from the keys is
//! bit for bit the pose sampled from those f32 frames.

/// A channel's frames.
#[derive(Clone, Debug, PartialEq)]
pub enum Samples {
    /// `frames × lanes` floats.
    F32(Vec<f32>),
    /// One sample (`lanes` floats) for every frame.
    Constant(Vec<f32>),
    /// Quantised keys with keyframe reduction, as cooked.
    Quantized(QuantizedKeys),
}

/// A quantised channel's keys ([`Samples::Quantized`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct QuantizedKeys {
    /// The kept frames, ascending from 0 to the clip's last frame; empty
    /// when every frame is a key.
    pub frames: Vec<u16>,
    /// Per key: 3 words (a rotation's smallest-three) or `lanes` words
    /// (unorm per lane).
    pub words: Vec<u16>,
    /// Per lane `min` and `extent`; empty for a rotation.
    pub range: Vec<f32>,
}

impl Default for Samples {
    fn default() -> Samples {
        Samples::F32(Vec::new())
    }
}

impl From<Vec<f32>> for Samples {
    fn from(v: Vec<f32>) -> Samples {
        Samples::F32(v)
    }
}

impl Samples {
    /// Bytes the frames take: what the cooked blob stored for the channel
    /// (its share of the `samples` and `packed` sections).
    pub fn bytes(&self) -> usize {
        match self {
            Samples::F32(v) | Samples::Constant(v) => v.len() * 4,
            Samples::Quantized(q) => q.frames.len() * 2 + q.words.len() * 2 + q.range.len() * 4,
        }
    }
}

pub(crate) fn words_per_key(lanes: u32, rotation: bool) -> usize {
    if rotation { 3 } else { lanes.max(1) as usize }
}

/// Where frame `i` lies among a quantised channel's keys: on key `k`, or
/// between keys `k` and `k + 1` at `u`.
enum KeyPos {
    On(usize),
    Between(usize, f32),
}

impl QuantizedKeys {
    fn locate(&self, i: usize) -> KeyPos {
        if self.frames.is_empty() {
            return KeyPos::On(i);
        }
        let i16 = i.min(usize::from(u16::MAX)) as u16;
        match self.frames.binary_search(&i16) {
            Ok(k) => KeyPos::On(k),
            Err(0) => KeyPos::On(0),
            Err(k) if k >= self.frames.len() => KeyPos::On(self.frames.len() - 1),
            Err(k) => {
                let (fa, fb) = (usize::from(self.frames[k - 1]), usize::from(self.frames[k]));
                KeyPos::Between(k - 1, (i - fa) as f32 / (fb - fa) as f32)
            }
        }
    }

    fn lane_of_key(&self, k: usize, lanes: usize, l: usize) -> f32 {
        dequantize(
            self.range[2 * l],
            self.range[2 * l + 1],
            self.words[k * lanes + l],
        )
    }

    fn quat_of_key(&self, k: usize) -> [f32; 4] {
        let w = &self.words[k * 3..k * 3 + 3];
        unpack_quat([w[0], w[1], w[2]])
    }

    /// Lane `l` of frame `i` of a channel of `lanes` lanes (not a rotation).
    pub(crate) fn lane(&self, lanes: usize, step: bool, i: usize, l: usize) -> f32 {
        match self.locate(i) {
            KeyPos::On(k) => self.lane_of_key(k, lanes, l),
            KeyPos::Between(k, u) => {
                let a = self.lane_of_key(k, lanes, l);
                if step {
                    return a;
                }
                let b = self.lane_of_key(k + 1, lanes, l);
                a + (b - a) * u
            }
        }
    }

    /// Frame `i` of a rotation channel, `[x, y, z, w]`.
    pub(crate) fn quat(&self, step: bool, i: usize) -> [f32; 4] {
        match self.locate(i) {
            KeyPos::On(k) => self.quat_of_key(k),
            KeyPos::Between(k, u) => {
                let a = self.quat_of_key(k);
                if step {
                    return a;
                }
                nlerp4(a, self.quat_of_key(k + 1), u)
            }
        }
    }
}

const SQRT1_2: f32 = std::f32::consts::FRAC_1_SQRT_2;
const Q15: f32 = 32767.0;

/// `q` scaled to unit length (identity for a zero or non-finite one).
pub fn normalize4(q: [f32; 4]) -> [f32; 4] {
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

/// Normalised lerp on the shorter arc, scalar and in a fixed order: how a
/// quantised rotation channel fills the frames between its keys.
pub fn nlerp4(a: [f32; 4], b: [f32; 4], u: f32) -> [f32; 4] {
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

/// A smallest-three quaternion (three words: bits 47..0, the largest
/// component's index in 46..45, then the others in order, 15 bits each)
/// back to a unit quaternion.
pub fn unpack_quat(w: [u16; 3]) -> [f32; 4] {
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

/// A unorm word over `[min, min + extent]`.
pub fn dequantize(min: f32, extent: f32, q: u16) -> f32 {
    min + extent * (f32::from(q) / 65535.0)
}
