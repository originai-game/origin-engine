//! The windowed-sinc resampler, `og.Audio.resampler` sinc (the default):
//! a voice's frame at a fractional source position is the sum of the
//! source frames around it weighted by a Kaiser-windowed sinc.
//!
//! - **Kernel**: `sinc(c·x)·c · I0(β√(1 − (x/W)²)) / I0(β)` for |x| < W
//!   source frames, with `W` = [`SINC_ZEROS`] (8) zero crossings each side
//!   and β = [`SINC_BETA`] (7, about 70 dB of stopband), normalised so the
//!   weights of each phase sum to 1 (DC passes exactly).
//! - **Pitch ≤ 1** (the step through the source at most one frame per
//!   output frame): `c` = 1, the source's Nyquist; [`SINC_TAPS`] (16) taps
//!   from a table of [`SINC_PHASES`] (2048) phases, the nearest taken (a
//!   time error of at most 1/4096 frame).
//! - **Pitch > 1**: the kernel widens by the step rounded up to the next
//!   eighth (`c` = 1/that, at or below the output's Nyquist), up to
//!   [`SINC_MAX_SPREAD`] (4, so at most 64 taps), from a table of
//!   [`WIDE_PHASES`] (512) phases built the first time a voice needs that
//!   width and shared after; past a step of 4 it aliases (as linear
//!   interpolation always does).
//! - **On a source frame** (the fraction exactly 0, as for every frame of
//!   an unpitched voice whose sound is at the mixer's rate) the frame is
//!   returned as it is: such a voice renders the same samples as the
//!   linear resampler, bit for bit, and costs no taps.
//!
//! A voice keeps the frames around its playhead in a 128-frame history,
//! filled one source frame at a time as the playhead passes it (the loop
//! seam included: the frames after `loopEnd − 1` are `loopStart`'s, as the
//! linear resampler reads them), so a frame costs one decoded source frame
//! and the taps. The tables are built once per process from `+ − × ÷` and
//! `sqrt` only (a series for sin and I0), so they are the same on every host.

use crate::sound::Reader;
use std::sync::OnceLock;

/// Zero crossings of the kernel on each side of the playhead.
pub const SINC_ZEROS: usize = 8;
/// Taps at a pitch up to 1.
pub const SINC_TAPS: usize = 2 * SINC_ZEROS;
/// The Kaiser window's β.
pub const SINC_BETA: f64 = 7.0;
/// Phases of the pitch ≤ 1 table.
pub const SINC_PHASES: usize = 2048;
/// The most the kernel widens for a pitched-up voice.
pub const SINC_MAX_SPREAD: f64 = 4.0;
/// Phases of a widened (pitch > 1) table.
pub const WIDE_PHASES: usize = 512;
/// Widths (spreads) are rounded up to this fraction.
const SPREAD_STEPS: f64 = 8.0;
/// Frames of history (a power of two).
const HISTORY: usize = 128;
const MASK: u64 = HISTORY as u64 - 1;
/// Frames kept behind the playhead when the history restarts: the widest
/// kernel's reach.
const BACK: u64 = 32;

/// sin(πt), from a Taylor series after reducing t to [-½, ½].
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

fn sinc(t: f64) -> f64 {
    if t == 0.0 {
        1.0
    } else {
        sin_pi(t) / (std::f64::consts::PI * t)
    }
}

/// The modified Bessel function I0, by its power series.
fn bessel_i0(x: f64) -> f64 {
    let (mut term, mut sum) = (1.0f64, 1.0f64);
    let q = x * x / 4.0;
    for k in 1..64 {
        term *= q / (k * k) as f64;
        sum += term;
        if term < sum * 1e-17 {
            break;
        }
    }
    sum
}

/// The windowed sinc at `x` source frames (cutoff at the source's Nyquist).
fn kernel(x: f64) -> f64 {
    let w = SINC_ZEROS as f64;
    if x.abs() >= w {
        return 0.0;
    }
    let r = x / w;
    sinc(x) * bessel_i0(SINC_BETA * (1.0 - r * r).max(0.0).sqrt()) / bessel_i0(SINC_BETA)
}

/// Per phase (0..=phases, fraction p / phases), the normalised weights of
/// the `2·reach` frames at offsets -(reach-1)..=reach from the playhead's
/// frame, for a kernel widened `spread` times.
fn polyphase(phases: usize, reach: usize, spread: f64) -> Vec<f32> {
    let mut out = Vec::with_capacity((phases + 1) * 2 * reach);
    for p in 0..=phases {
        let frac = p as f64 / phases as f64;
        let w: Vec<f64> = (0..2 * reach)
            .map(|j| kernel((j as f64 - (reach - 1) as f64 - frac) / spread))
            .collect();
        let sum: f64 = w.iter().sum();
        out.extend(w.iter().map(|x| (x / sum) as f32));
    }
    out
}

/// The pitch ≤ 1 table: [`SINC_PHASES`] + 1 rows of [`SINC_TAPS`].
fn narrow() -> &'static [f32] {
    static TABLE: OnceLock<Vec<f32>> = OnceLock::new();
    TABLE.get_or_init(|| polyphase(SINC_PHASES, SINC_ZEROS, 1.0))
}

/// A widened table: its taps per phase and its weights.
pub(crate) struct Wide {
    reach: usize,
    weights: Vec<f32>,
}

/// The table of width bucket `k` (spread 1 + k/8), built on first use.
fn wide(k: usize) -> std::sync::Arc<Wide> {
    use std::sync::{Arc, Mutex};
    static WIDE: Mutex<Vec<Option<Arc<Wide>>>> = Mutex::new(Vec::new());
    let mut all = WIDE.lock().unwrap_or_else(|e| e.into_inner());
    if all.len() <= k {
        all.resize(k + 1, None);
    }
    all[k]
        .get_or_insert_with(|| {
            let spread = 1.0 + k as f64 / SPREAD_STEPS;
            let reach = (SINC_ZEROS as f64 * spread).ceil() as usize;
            Arc::new(Wide {
                reach,
                weights: polyphase(WIDE_PHASES, reach, spread),
            })
        })
        .clone()
}

/// A voice's loop region, as its playhead wraps.
#[derive(Clone, Copy)]
pub(crate) struct Region {
    pub looping: bool,
    pub start: u64,
    pub end: u64,
}

impl Region {
    /// The source frame `n` frames after `i`, across the loop seam.
    fn after(&self, i: u64, n: u64) -> u64 {
        let s = i + n;
        if self.looping && s >= self.end && self.end > self.start {
            self.start + (s - self.end) % (self.end - self.start)
        } else {
            s
        }
    }
}

/// The source frames around a voice's playhead, in the order it plays them.
pub(crate) struct History {
    /// The ring, written twice (at `n` and `n + HISTORY`) so any window of
    /// up to HISTORY frames is one contiguous slice.
    ring: Box<[[f32; 2]; 2 * HISTORY]>,
    /// The virtual index (frames pushed, ever, plus a base) of the next push.
    end: u64,
    /// The source frame the next push reads.
    src: u64,
    /// The virtual index and the source frame of the playhead's frame.
    vi: u64,
    i: u64,
    valid: bool,
    /// The widened table in use and its width bucket.
    wide: Option<(usize, std::sync::Arc<Wide>)>,
    /// The step the bucket and reach were worked out for.
    step: f64,
    bucket: usize,
    reach: u64,
}

impl History {
    pub fn new() -> History {
        History {
            ring: Box::new([[0.0; 2]; 2 * HISTORY]),
            end: 0,
            src: 0,
            vi: 0,
            i: 0,
            valid: false,
            wide: None,
            step: 1.0,
            bucket: 0,
            reach: SINC_ZEROS as u64,
        }
    }

    /// The playhead jumped (a start, a cue, a resync): refill around it.
    pub fn invalidate(&mut self) {
        self.valid = false;
    }

    fn push(&mut self, frame: (f32, f32)) {
        let at = (self.end & MASK) as usize;
        self.ring[at] = [frame.0, frame.1];
        self.ring[at + HISTORY] = [frame.0, frame.1];
        self.end += 1;
    }

    /// The voice's frame at source position `i + frac` moving `step` source
    /// frames per output frame.
    pub fn frame(
        &mut self,
        reader: &mut Reader,
        region: Region,
        i: u64,
        frac: f32,
        step: f64,
    ) -> (f32, f32) {
        if step != self.step {
            // Width bucket: the spread rounded up to an eighth (0: pitch ≤ 1).
            self.step = step;
            self.bucket = if step <= 1.0 {
                0
            } else {
                ((step.min(SINC_MAX_SPREAD) - 1.0) * SPREAD_STEPS).ceil() as usize
            };
            self.reach =
                (SINC_ZEROS as f64 * (1.0 + self.bucket as f64 / SPREAD_STEPS)).ceil() as u64;
        }
        let (bucket, reach) = (self.bucket, self.reach);
        if self.valid && i != self.i {
            if i > self.i {
                self.vi += i - self.i;
            } else if region.looping && step < (region.end - region.start) as f64 {
                // Wrapped: the rest of the region, then from its start.
                self.vi += (region.end - self.i) + (i - region.start);
            } else {
                self.valid = false;
            }
            self.i = i;
        }
        if !self.valid {
            // Restart: BACK frames behind (silence before the sound), then on.
            self.vi = BACK + HISTORY as u64;
            self.i = i;
            self.end = self.vi - BACK;
            for k in (1..=BACK).rev() {
                let f = if i >= k {
                    reader.frame(i - k)
                } else {
                    (0.0, 0.0)
                };
                self.push(f);
            }
            self.src = i;
            self.valid = true;
        }
        let need = self.vi + reach + 1;
        if need > self.end + HISTORY as u64 - BACK {
            // A step past the history: skip the frames no tap reads.
            let skip = need - self.end - (HISTORY as u64 - BACK);
            self.src = region.after(self.src, skip);
            self.end += skip;
        }
        while self.end < need {
            let f = reader.frame(self.src);
            self.push(f);
            self.src = region.after(self.src, 1);
        }
        if frac == 0.0 {
            // On a source frame: the frame itself.
            let f = self.ring[(self.vi & MASK) as usize];
            return (f[0], f[1]);
        }
        let taps = 2 * reach as usize;
        let row = if bucket == 0 {
            let phase = ((frac * SINC_PHASES as f32 + 0.5) as usize).min(SINC_PHASES);
            &narrow()[phase * taps..(phase + 1) * taps]
        } else {
            if self.wide.as_ref().is_none_or(|w| w.0 != bucket) {
                self.wide = Some((bucket, wide(bucket)));
            }
            let w = &self.wide.as_ref().expect("just set").1;
            debug_assert_eq!(w.reach * 2, taps);
            let phase = ((frac * WIDE_PHASES as f32 + 0.5) as usize).min(WIDE_PHASES);
            &w.weights[phase * taps..(phase + 1) * taps]
        };
        let first = ((self.vi + 1 - reach) & MASK) as usize;
        let window = &self.ring[first..first + taps];
        let (mut l, mut r) = (0.0f32, 0.0f32);
        for (w, f) in row.iter().zip(window) {
            l += w * f[0];
            r += w * f[1];
        }
        (l, r)
    }
}
