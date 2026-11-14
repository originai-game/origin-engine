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
//! - **The sum**: a frame's taps in pairs (2p, 2p + 1), left and right of
//!   both as four lanes, summed lane by lane from the first pair to the
//!   last; then lanes 0 + 2 are the left and 1 + 3 the right. One SSE add
//!   carries a pair on x86_64, and other targets (wasm) do the same
//!   operations a lane at a time, so the samples are the same on every host.
//!   Eight frames' sums run side by side, so their adds do not wait on one
//!   another.
//!
//! A voice keeps the frames around its playhead in a 128-frame history
//! (the loop seam included: the frames after `loopEnd − 1` are
//! `loopStart`'s, as the linear resampler reads them), rendered in
//! segments: the frames of a run whose windows the history holds together,
//! with the history filled once for them, from the decoded block at hand,
//! so a frame costs about one decoded source frame and the taps. The
//! tables are built once per process from `+ − × ÷` and `sqrt` only (a
//! series for sin and I0), so they are the same on every host.

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

    /// Pushes source frames until `end` reaches at least `need`, and on
    /// up to `most` while they are in the block at hand.
    #[inline]
    fn fill(&mut self, reader: &mut Reader, region: Region, need: u64, most: u64) {
        while self.end < need {
            // The frames before the loop seam.
            let straight = if region.looping && region.end > region.start && self.src < region.end {
                region.end - self.src
            } else {
                u64::MAX
            };
            let (ring, end) = (&mut self.ring, &mut self.end);
            let n = reader.frames(self.src, straight.min(most - *end), |f| {
                let at = (*end & MASK) as usize;
                ring[at] = [f.0, f.1];
                ring[at + HISTORY] = [f.0, f.1];
                *end += 1;
            });
            self.src = region.after(self.src, n);
        }
    }

    /// The playhead's virtual index at source frame `i` if it got there by
    /// playing on (forwards, or wrapped at the loop seam); `None` when it
    /// jumped and the history restarts.
    #[inline]
    fn follow(&self, region: Region, i: u64, step: f64) -> Option<u64> {
        if !self.valid {
            None
        } else if i >= self.i {
            Some(self.vi + (i - self.i))
        } else if region.looping && step < (region.end - region.start) as f64 {
            // Wrapped: the rest of the region, then from its start.
            Some(self.vi + (region.end - self.i) + (i - region.start))
        } else {
            None
        }
    }

    /// Restarts at source frame `i`: BACK frames behind (silence before
    /// the sound), then on.
    fn restart(&mut self, reader: &mut Reader, i: u64) {
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

    /// The voice's frames at the source positions `at` (frame, fraction),
    /// moving `step` source frames per output frame, into `out`.
    ///
    /// In segments: the playhead is followed over as many frames as the
    /// ring holds the windows of together, the ring is filled once for
    /// them, then each window is summed.
    pub fn render(
        &mut self,
        reader: &mut Reader,
        region: Region,
        at: &[(u64, f32)],
        step: f64,
        out: &mut [(f32, f32)],
    ) {
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
        let bucket = self.bucket;
        if bucket != 0 && self.wide.as_ref().is_none_or(|w| w.0 != bucket) {
            self.wide = Some((bucket, wide(bucket)));
        }
        // Out of `self` while the ring fills, and back after.
        let wide = if bucket != 0 { self.wide.take() } else { None };
        self.render_with(reader, region, at, step, out, wide.as_ref().map(|w| &*w.1));
        if wide.is_some() {
            self.wide = wide;
        }
    }

    fn render_with(
        &mut self,
        reader: &mut Reader,
        region: Region,
        at: &[(u64, f32)],
        step: f64,
        out: &mut [(f32, f32)],
        wide: Option<&Wide>,
    ) {
        let reach = self.reach;
        let (table, phases) = match wide {
            None => (narrow(), SINC_PHASES),
            Some(w) => {
                debug_assert_eq!(w.reach as u64, reach);
                (&w.weights[..], WIDE_PHASES)
            }
        };
        let taps = 2 * reach as usize;
        // The most a segment's windows span, so a skip (below) keeps them.
        let span = HISTORY as u64 - BACK;
        let mut j = 0;
        while j < at.len() {
            let i0 = at[j].0;
            match self.follow(region, i0, step) {
                Some(vi) => self.vi = vi,
                None => self.restart(reader, i0),
            }
            // The segment: the frames played on from `i0` (no wrap) whose
            // windows the ring holds together.
            let reachable = i0 + span - 2 * reach;
            let mut k = j + 1;
            while k < at.len() && at[k].0 >= at[k - 1].0 && at[k].0 <= reachable {
                k += 1;
            }
            let last = at[k - 1].0;
            let vi = self.vi;
            self.vi = vi + (last - i0);
            self.i = last;
            let need = self.vi + reach + 1;
            if need > self.end + span {
                // A step past the history: skip the frames no tap reads.
                let skip = need - self.end - span;
                self.src = region.after(self.src, skip);
                self.end += skip;
            }
            if self.end < need {
                // What the block at hand holds is read ahead, as far as the
                // ring keeps every frame a window may still read: the
                // segment's and the widest kernel's around the playhead (a
                // later render may be pitched further up).
                self.fill(reader, region, need, vi + 1 - BACK + HISTORY as u64);
            }
            // Ring slot of source frame `i`'s window, and of `i` itself.
            let window = (vi + 1 - reach).wrapping_sub(i0);
            let centre = vi.wrapping_sub(i0);
            let mut group = [(0usize, 0usize, 0usize); GROUP];
            let mut queued = 0;
            for (o, &(i, frac)) in at[j..k].iter().enumerate() {
                if frac == 0.0 {
                    // On a source frame: the frame itself.
                    let f = self.ring[(centre.wrapping_add(i) & MASK) as usize];
                    out[j + o] = (f[0], f[1]);
                    continue;
                }
                // Through i32 (one instruction; the value is in [0.5, phases + 0.5]).
                let phase = ((frac * phases as f32 + 0.5) as i32 as usize).min(phases);
                group[queued] = (
                    j + o,
                    (window.wrapping_add(i) & MASK) as usize,
                    phase * taps,
                );
                queued += 1;
                if queued == GROUP {
                    sum(&self.ring, table, taps, &group, out);
                    queued = 0;
                }
            }
            sum(&self.ring, table, taps, &group[..queued], out);
            j = k;
        }
    }
}

/// Frames whose taps are summed at once: independent sums, so the adds of
/// one do not wait for another's.
const GROUP: usize = 8;

/// Each queued frame's windowed sum, into `out`. The even taps and the odd
/// taps sum in two partial sums, each from the first tap to the last, then
/// the odd sum joins the even: four lanes (left and right of a tap pair) a
/// vector add carries, in an order fixed here, so every host and every
/// grouping gives the same samples.
#[inline]
fn sum(
    ring: &[[f32; 2]; 2 * HISTORY],
    table: &[f32],
    taps: usize,
    queued: &[(usize, usize, usize)],
    out: &mut [(f32, f32)],
) {
    debug_assert!(taps % 2 == 0);
    let flat = ring.as_flattened();
    let rows = |row: usize| table[row..][..taps].as_chunks::<2>().0;
    let windows = |first: usize| flat[2 * first..][..2 * taps].as_chunks::<4>().0;
    if let Ok(group) = <&[_; GROUP]>::try_from(queued) {
        let acc = dot::<GROUP>(
            std::array::from_fn(|k| rows(group[k].2)),
            std::array::from_fn(|k| windows(group[k].1)),
        );
        for (q, a) in group.iter().zip(acc) {
            out[q.0] = (a[0] + a[2], a[1] + a[3]);
        }
        return;
    }
    for &(j, first, row) in queued {
        let [a] = dot([rows(row)], [windows(first)]);
        out[j] = (a[0] + a[2], a[1] + a[3]);
    }
}

/// `N` frames' tap pairs summed lane by lane: for each frame, the left and
/// right of its even taps (lanes 0, 1) and of its odd taps (lanes 2, 3).
/// SSE2 on x86_64 (four lanes, one add); elsewhere the same operations a
/// lane at a time, to the same bits.
#[inline(always)]
fn dot<const N: usize>(rows: [&[[f32; 2]]; N], windows: [&[[f32; 4]]; N]) -> [[f32; 4]; N] {
    let pairs = rows[0].len();
    assert!(rows.iter().all(|r| r.len() == pairs) && windows.iter().all(|w| w.len() == pairs));
    #[cfg(target_arch = "x86_64")]
    // SAFETY: SSE and SSE2 are in x86_64's baseline; each load reads the 8
    // bytes of a `[f32; 2]` or the 16 of a `[f32; 4]` it is given, and the
    // store writes the 16 bytes of `lanes`.
    unsafe {
        use std::arch::x86_64::*;
        let mut acc = [_mm_setzero_ps(); N];
        for t in 0..pairs {
            for k in 0..N {
                let w = _mm_castpd_ps(_mm_load_sd(rows[k][t].as_ptr().cast()));
                let x = _mm_loadu_ps(windows[k][t].as_ptr());
                acc[k] = _mm_add_ps(acc[k], _mm_mul_ps(_mm_unpacklo_ps(w, w), x));
            }
        }
        acc.map(|a| {
            let mut lanes = [0.0f32; 4];
            _mm_storeu_ps(lanes.as_mut_ptr(), a);
            lanes
        })
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        let mut acc = [[0.0f32; 4]; N];
        for t in 0..pairs {
            for k in 0..N {
                let (w, x) = (rows[k][t], windows[k][t]);
                let w = [w[0], w[0], w[1], w[1]];
                for m in 0..4 {
                    acc[k][m] += w[m] * x[m];
                }
            }
        }
        acc
    }
}
