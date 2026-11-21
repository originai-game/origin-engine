//! Automatic quality: from the measured frame time it moves the render
//! scale first, then the graphics preset, down when frames run over their
//! budget and back up when there is room, with hysteresis so that it never
//! oscillates. Pure logic: the host feeds it one [`FrameSample`] per drawn
//! frame and applies the [`AutoQuality::scale`] and [`AutoQuality::step`]
//! it answers (og-runtime's `display.rs`); it reads no clock and draws
//! nothing.
//!
//! The ladder is one list of rungs, best first ([`Rung`]): the render scale
//! from the most to the least in steps of [`SCALE_STEP`], then, at the
//! least scale, each preset step the host offers (a cap at medium, then at
//! low). A rung is what the frame draws there: its scale, its preset step
//! and the upscaler the frame resolves with (the game's upscaler under that
//! step's cap, as the renderer draws it at that scale:
//! [`Upscaler::drawn`]), so `auto` is FSR 3.1 on every rung below scale 1
//! and TAAU at it. A step down goes to the next rung down and a step up to
//! the next one up, so the scale goes first on the way down and comes back
//! last on the way up.
//!
//! A rung's cost is the pixels' cost times its scale's area and its preset
//! factor, plus its upscaler's fixed cost: the resolve runs at the output's
//! size whatever the scale, and FSR 3.1's is several times TAAU's (on an
//! NVIDIA L4 at 1080p TAAU's resolve is [`UPSCALE_SHARE`]'s first share of
//! a frame, FSR's [`FSR_PER_TAAU`] times it). The upscaler's time is
//! measured where the GPU times its passes ([`FrameSample::upscale_ms`],
//! each upscaler's own, the other's estimated from it by that ratio), else
//! taken as its share of the frame. A rung that costs no less than the one
//! above it (scale 1's TAAU against 0.9's FSR on a GPU whose frame is
//! mostly the resolve) is dominated: it is passed over both ways, so a
//! step down never lands where it only adds the upscaler's cost.
//!
//! Hysteresis, two ways:
//!
//! - With the GPU's own time ([`FrameSample::gpu_ms`], timestamp queries),
//!   it steps down when the smoothed time passes [`DOWN_AT`] of the budget
//!   for [`DOWN_FRAMES`] frames in a row, and up only when the time the
//!   rung above would take (the scale's pixels, or [`PRESET_COST`] more for
//!   a preset step), padded by [`SPREAD_SIGMAS`] of the measured spread of
//!   the frame times, stays under [`UP_AT`] for [`UP_FRAMES`] frames. A
//!   noisy scene whose rung above is predicted just under the up line is
//!   left where it is: the prediction is a mean, and the frames around it
//!   would cross the down line. A step up that steps back down within
//!   [`VERIFY_FRAMES`] fails as a trial does below: that rung waits twice
//!   as long before it is tried again, and is given up after
//!   [`MAX_FAILURES`] (studio-win, G1-4: a rung predicted at 13.0 to 13.5
//!   ms against an up line of 13.4 measured 14 to 15 ms, and content
//!   spikes took it back down, one cycle every 30 s).
//! - Without it (WebGPU without timestamp queries: Safari, every publish
//!   build), the interval between frames is all there is: a frame that
//!   missed its vsync shows as an interval past [`LATE_AT`] of the budget
//!   (unless the CPU's own work explains it, when a lower scale would not
//!   help and nothing moves). Room cannot be seen at a vsync-locked rate,
//!   so a step up is a trial after [`TRIAL_FRAMES`] calm frames: a trial
//!   that runs late within [`VERIFY_FRAMES`] goes back down, waits twice as
//!   long for the next trial of that rung, and after [`MAX_FAILURES`]
//!   failed trials that rung is not tried again until the settings or the
//!   budget change. So a rung is left and re-entered a bounded number of
//!   times.
//!
//! A step down must pay for itself. Each run of steps down in a row is
//! measured against the rung it began from: when the cost it saved is less
//! than [`HELP_AT`] of what the pixels (or [`PRESET_COST`]) predict, the
//! frame is not bound by them (vertex or CPU-side work the CPU time does not
//! show, or the browser's own), so fewer pixels only degrade the image. It
//! goes back to that rung and steps down no further until the bounds change
//! or the cost there moves past [`REPROBE_AT`] either way (the load changed;
//! the next step down is measured again). The frame interval moves in whole
//! vsyncs, so without GPU time a run is judged only once its predicted
//! saving reaches the budget (The Last Light on Chrome, 2560×1440, ~100 ms
//! frames: 1.0 → 0.9 → 0.8 saved 19 and 18 ms, 0.7 cost 3 ms more, and it
//! went on to 0.5 and a preset step for nothing).

use crate::{QualityPreset, Upscaler};

/// The render scale's step on the ladder.
pub const SCALE_STEP: f32 = 0.1;
/// Steps down past this fraction of the budget (GPU time).
pub const DOWN_AT: f64 = 0.9;
/// Steps up when the rung above would stay under this fraction (GPU time).
pub const UP_AT: f64 = 0.75;
/// A frame's interval past this fraction of the budget is late (no GPU time).
pub const LATE_AT: f64 = 1.2;
/// Standard deviations of the measured GPU time a step up's prediction is
/// padded by (GPU time): room is room for the frames around the mean too.
pub const SPREAD_SIGMAS: f64 = 2.0;
/// The cost a preset step adds back, as a fraction, when the GPU time
/// predicts a step up.
pub const PRESET_COST: f64 = 0.3;
/// FSR 3.1's resolve below scale 1 against TAAU's at it, both at the
/// output's size: NVIDIA L4 at 1080p (og-tests `fsr::cost`, 2026-11-21),
/// FSR at 0.9 against TAAU at 1 is 1.03 / 0.26 ms on the grass scene
/// (3.9x) and 0.78 / 0.16 ms on thin geometry (5.0x); at 0.77, 3.1x and
/// 4.3x.
pub const FSR_PER_TAAU: f64 = 4.0;
/// The share of a frame TAAU's (at scale 1) and FSR 3.1's (below it)
/// resolves take where the GPU times no pass: the L4's grass scene at
/// 1080p, 0.26 of 13.1 ms and 1.03 of 13.4 ms (`fsr::cost`).
pub const UPSCALE_SHARE: [f64; 2] = [0.02, 0.08];
/// Frames in a row over the budget before a step down.
pub const DOWN_FRAMES: u32 = 20;
/// Frames in a row with room before a step up (GPU time).
pub const UP_FRAMES: u32 = 120;
/// Calm frames before a trial step up (no GPU time), doubled per failure;
/// with GPU time the wait is [`UP_FRAMES`], doubled per failure the same way.
pub const TRIAL_FRAMES: u32 = 300;
/// Frames after a trial step up in which running late fails it.
pub const VERIFY_FRAMES: u32 = 180;
/// Failed trials before a rung is not tried again.
pub const MAX_FAILURES: u32 = 3;
/// An interval longer than this is a pause (a hidden page, a breakpoint),
/// not a frame's cost.
pub const PAUSE_MS: f64 = 1000.0;
/// Frames after any change before the next decision (the new rung's cost
/// settles: targets remade, pipelines compiled).
pub const SETTLE_FRAMES: u32 = 30;
/// A run of steps down that saved less than this share of its predicted
/// saving did not help.
pub const HELP_AT: f64 = 0.5;
/// The factor, either way, by which the cost at a rung a run of steps down
/// returned to must move before it steps down again.
pub const REPROBE_AT: f64 = 1.5;

/// One drawn frame, as the host measured it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameSample {
    /// Milliseconds since the last drawn frame.
    pub interval_ms: f64,
    /// Milliseconds the engine's frame took on the CPU (ticks, extraction,
    /// encoding and submission).
    pub cpu_ms: f64,
    /// Milliseconds the GPU spent on the frame, where timestamp queries
    /// measure it (they arrive a frame or two late; the newest is fine).
    pub gpu_ms: Option<f64>,
    /// Milliseconds of that the upscaler's resolve took (the `taa` and
    /// `taa-sharpen` passes), where measured.
    pub upscale_ms: Option<f64>,
}

/// The bounds the controller moves within.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AutoQualityBounds {
    /// The render scale at the top rung (the game's render scale).
    pub max_scale: f32,
    /// The least render scale it lowers to.
    pub min_scale: f32,
    /// Preset steps it may take below the game's own quality, once the
    /// scale is at its least (0: none).
    pub preset_steps: u8,
    /// Milliseconds a frame may take: the frame cap's interval, or the
    /// display's at no cap.
    pub budget_ms: f64,
    /// The game's upscaler (`RenderQuality::upscaler`) where the frame
    /// resolves under TAA; `None` without TAA (a scaled frame is then
    /// stretched, at no fixed cost).
    pub upscaler: Option<Upscaler>,
    /// The device draws FSR 3.1 ([`crate::Caps::COMPUTE`]).
    pub fsr: bool,
}

/// One rung of the ladder ([module docs](self)): what a frame draws there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rung {
    pub scale: f32,
    /// Preset steps below the game's quality ([`step_preset`]).
    pub step: u8,
    /// The upscaler the frame resolves with (never `Auto`); `None` without TAA.
    pub upscaler: Option<Upscaler>,
}

/// The preset a step caps the game's quality at: medium, then low.
pub fn step_preset(step: u8) -> Option<QualityPreset> {
    match step {
        0 => None,
        1 => Some(QualityPreset::Medium),
        _ => Some(QualityPreset::Low),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Trial {
    /// The rung tried.
    rung: usize,
    frames_left: u32,
}

/// The controller ([module docs](self)).
#[derive(Clone, Debug)]
pub struct AutoQuality {
    bounds: AutoQualityBounds,
    /// The ladder, best first.
    rungs: Vec<Rung>,
    rung: usize,
    /// Each upscaler's smoothed resolve time (TAAU, FSR 3.1), once measured.
    fixed: [Option<f64>; 2],
    /// The smoothed cost (GPU ms, or the interval) and CPU time; `None`
    /// until the first sample after a change.
    cost: Option<f64>,
    /// The smoothed squared deviation of the cost from its smoothed value.
    spread: f64,
    cpu: f64,
    over: u32,
    calm: u32,
    settle: u32,
    trial: Option<Trial>,
    /// Failed trials per rung.
    failures: Vec<u32>,
    /// The rung the steps down in a row began from (the last that helped)
    /// and its cost; `None` once a frame is not over.
    helped: Option<(usize, f64)>,
    /// The rung steps down stop at, and its cost, after a run that did not
    /// help.
    floor: Option<(usize, f64)>,
}

impl AutoQuality {
    /// A controller at the top rung (the game's scale and quality).
    pub fn new(bounds: AutoQualityBounds) -> AutoQuality {
        let mut q = AutoQuality {
            bounds,
            rungs: Vec::new(),
            rung: 0,
            fixed: [None; 2],
            cost: None,
            spread: 0.0,
            cpu: 0.0,
            over: 0,
            calm: 0,
            settle: 0,
            trial: None,
            failures: Vec::new(),
            helped: None,
            floor: None,
        };
        q.set_bounds(bounds);
        q
    }

    /// New bounds (settings or budget changed): the rung keeps its scale
    /// and step where they are still on the ladder (else the nearest), and
    /// every failed trial and step down that did not help is forgotten.
    pub fn set_bounds(&mut self, bounds: AutoQualityBounds) {
        let (scale, step) = if self.rungs.is_empty() {
            (f32::INFINITY, 0)
        } else {
            (self.scale(), self.step())
        };
        self.bounds = AutoQualityBounds {
            max_scale: finite_or(bounds.max_scale, 1.0),
            min_scale: finite_or(bounds.min_scale, 1.0),
            ..bounds
        };
        let b = self.bounds;
        let scales = ladder(b.max_scale, b.min_scale);
        let least = scales[scales.len() - 1];
        let rung = |scale: f32, step: u8| Rung {
            scale,
            step,
            upscaler: b.upscaler.map(|u| {
                let u = step_preset(step).map_or(u, |p| u.capped(p));
                let render = if scale < 1.0 - 1e-3 {
                    std::cmp::Ordering::Less
                } else if scale > 1.0 + 1e-3 {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Equal
                };
                u.drawn(render, b.fsr)
            }),
        };
        self.rungs = scales.iter().map(|s| rung(*s, 0)).collect();
        self.rungs
            .extend((1..=b.preset_steps).map(|step| rung(least, step)));
        let rungs = self.rungs.len();
        self.rung = if step > 0 {
            (scales.len() - 1 + usize::from(step)).min(rungs - 1)
        } else {
            scales
                .iter()
                .position(|s| *s <= scale + 1e-4)
                .unwrap_or(scales.len() - 1)
        };
        self.failures = vec![0; rungs];
        self.helped = None;
        self.floor = None;
        self.restart();
    }

    /// The bounds in force.
    pub fn bounds(&self) -> AutoQualityBounds {
        self.bounds
    }

    /// The rung drawn now.
    pub fn rung(&self) -> Rung {
        self.rungs[self.rung]
    }

    /// The render scale to draw at.
    pub fn scale(&self) -> f32 {
        self.rung().scale
    }

    /// The preset steps taken below the game's quality (0: none).
    pub fn step(&self) -> u8 {
        self.rung().step
    }

    /// The smoothed cost the decisions read, ms (GPU time, else the frame
    /// interval); 0 before a sample.
    pub fn cost_ms(&self) -> f64 {
        self.cost.unwrap_or(0.0)
    }

    fn restart(&mut self) {
        self.cost = None;
        self.spread = 0.0;
        self.over = 0;
        self.calm = 0;
        self.settle = SETTLE_FRAMES;
    }

    /// One drawn frame. True when the scale or the step changed.
    pub fn sample(&mut self, s: FrameSample) -> bool {
        let budget = self.bounds.budget_ms;
        // A hidden page or a pause says nothing about the rung's cost.
        if budget.is_nan()
            || budget <= 0.0
            || !s.interval_ms.is_finite()
            || s.interval_ms > PAUSE_MS
        {
            return false;
        }
        if self.settle > 0 {
            self.settle -= 1;
            return false;
        }
        let gpu = s.gpu_ms.filter(|g| g.is_finite() && *g >= 0.0);
        if let (Some(u), Some(ms)) = (
            self.rung().upscaler,
            s.upscale_ms.filter(|m| m.is_finite() && *m >= 0.0),
        ) {
            let f = &mut self.fixed[upscale_index(u)];
            *f = Some(f.map_or(ms, |f| f * 0.9 + ms * 0.1));
        }
        let cost = gpu.unwrap_or(s.interval_ms);
        let smoothed = match self.cost {
            Some(c) => {
                let d = cost - c;
                self.spread = self.spread * 0.9 + d * d * 0.1;
                c * 0.9 + cost * 0.1
            }
            None => {
                self.cpu = s.cpu_ms;
                cost
            }
        };
        self.cost = Some(smoothed);
        self.cpu = self.cpu * 0.9 + s.cpu_ms * 0.1;
        let over = match gpu {
            Some(_) => smoothed > budget * DOWN_AT,
            // Late, and not for the CPU's own work.
            None => smoothed > budget * LATE_AT && self.cpu < budget * 0.75,
        };
        if let Some(t) = &mut self.trial {
            t.frames_left = t.frames_left.saturating_sub(1);
            if t.frames_left == 0 {
                self.trial = None;
            }
        }
        if over {
            self.calm = 0;
            self.over += 1;
            if self.over < DOWN_FRAMES {
                return false;
            }
            // The interval moves in whole vsyncs: a smaller predicted
            // saving may not show in it yet.
            let quantum = if gpu.is_some() { 0.0 } else { budget };
            return self.step_down(smoothed, quantum);
        }
        self.over = 0;
        self.helped = None;
        if self.rung == 0 {
            return false;
        }
        // The next rung up that the one above it does not dominate.
        let mut up = self.rung - 1;
        while up > 0 && self.dominated(smoothed, up) {
            up -= 1;
        }
        let room = match gpu {
            Some(_) => self.predicted(up) < budget * UP_AT,
            None => smoothed <= budget * 1.05,
        };
        if !room {
            self.calm = 0;
            return false;
        }
        self.calm += 1;
        // A rung that failed its trials is given up; each failure doubles
        // the wait before the next, on both paths.
        if self.failures[up] >= MAX_FAILURES {
            return false;
        }
        let wait = match gpu {
            Some(_) => UP_FRAMES,
            None => TRIAL_FRAMES,
        } << self.failures[up];
        if self.calm < wait {
            return false;
        }
        self.rung = up;
        // Every step up is a trial: back down within VERIFY_FRAMES fails it.
        self.trial = Some(Trial {
            rung: up,
            frames_left: VERIFY_FRAMES + SETTLE_FRAMES,
        });
        self.restart();
        true
    }

    /// Frames over the budget for long enough at `cost`: first the run of
    /// steps down so far is judged (once its predicted saving reaches
    /// `quantum`), and one that did not help goes back to where it began;
    /// else one rung down, unless the floor or the ladder's end holds it.
    fn step_down(&mut self, cost: f64, quantum: f64) -> bool {
        if let Some((from, was)) = self.helped {
            let predicted = was - self.cost_at(was, from, self.rung);
            if predicted >= quantum {
                if was - cost < HELP_AT * predicted {
                    self.floor = Some((from, was));
                    self.helped = None;
                    self.trial = None;
                    self.rung = from;
                    self.restart();
                    return true;
                }
                self.helped = None;
            }
        }
        if let Some((rung, at)) = self.floor
            && self.rung >= rung
        {
            if cost < at * REPROBE_AT && cost * REPROBE_AT > at {
                return false;
            }
            self.floor = None;
        }
        // The next rung down that saves anything (a dominated one only
        // adds its upscaler's cost).
        let last = self.rungs.len() - 1;
        let mut down = self.rung + 1;
        while down < last && self.cost_at(cost, self.rung, down) >= cost {
            down += 1;
        }
        if down > last || self.cost_at(cost, self.rung, down) >= cost {
            return false;
        }
        if let Some(t) = self.trial.take()
            && t.rung == self.rung
        {
            self.failures[t.rung] += 1;
        }
        self.helped.get_or_insert((self.rung, cost));
        self.rung = down;
        self.restart();
        true
    }

    /// The ms the upscaler `u` resolves in, for a frame of `cost` ms: its
    /// measured time, else the other's times [`FSR_PER_TAAU`] (or over it),
    /// else its [`UPSCALE_SHARE`] of the frame.
    fn fixed_ms(&self, u: Option<Upscaler>, cost: f64) -> f64 {
        let Some(u) = u else { return 0.0 };
        let k = upscale_index(u);
        let ratio = if k == 1 {
            FSR_PER_TAAU
        } else {
            1.0 / FSR_PER_TAAU
        };
        self.fixed[k]
            .or_else(|| self.fixed[1 - k].map(|o| o * ratio))
            .unwrap_or(UPSCALE_SHARE[k] * cost)
    }

    /// The cost rung `to` is predicted to take where rung `from` costs
    /// `cost`: the pixels' part by the scales' areas and
    /// 1 + [`PRESET_COST`] per preset step, plus `to`'s upscaler's fixed
    /// cost in place of `from`'s.
    fn cost_at(&self, cost: f64, from: usize, to: usize) -> f64 {
        let (a, b) = (self.rungs[from], self.rungs[to]);
        let area = f64::from(b.scale / a.scale).powi(2);
        let preset = (1.0 + PRESET_COST).powi(i32::from(a.step) - i32::from(b.step));
        let pixels = (cost - self.fixed_ms(a.upscaler, cost)).max(0.0);
        pixels * area * preset + self.fixed_ms(b.upscaler, cost)
    }

    /// Whether rung `k` costs no less than the one above it, at a frame of
    /// `cost` ms here.
    fn dominated(&self, cost: f64, k: usize) -> bool {
        self.cost_at(cost, self.rung, k) >= self.cost_at(cost, self.rung, k - 1)
    }

    /// What the smoothed cost would be on rung `to`, above this one,
    /// padded by [`SPREAD_SIGMAS`] of the measured spread (GPU time).
    fn predicted(&self, to: usize) -> f64 {
        let cost = self.cost_ms() + SPREAD_SIGMAS * self.spread.max(0.0).sqrt();
        self.cost_at(cost, self.rung, to)
    }
}

/// [`AutoQuality::fixed`]'s index for an upscaler drawn (never `Auto`).
fn upscale_index(u: Upscaler) -> usize {
    usize::from(u == Upscaler::Fsr3)
}

fn finite_or(x: f32, or: f32) -> f32 {
    if x.is_finite() { x } else { or }
}

/// The ladder's scales from `max` down to `min` (each within the render
/// scale's bounds): `max`, the multiples of [`SCALE_STEP`] below it, `min`.
fn ladder(max: f32, min: f32) -> Vec<f32> {
    let max = max.clamp(crate::MIN_RENDER_SCALE, crate::MAX_RENDER_SCALE);
    let min = min.clamp(crate::MIN_RENDER_SCALE, max);
    let mut out = vec![max];
    let mut k = (max / SCALE_STEP - 1e-3).floor() as i32;
    loop {
        let s = k as f32 * SCALE_STEP;
        if s < min + 1e-3 {
            break;
        }
        if s < max - 1e-3 {
            out.push((s * 100.0).round() / 100.0);
        }
        k -= 1;
    }
    if (out[out.len() - 1] - min).abs() > 1e-3 {
        out.push(min);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUDGET: f64 = 1000.0 / 60.0;

    fn bounds(steps: u8) -> AutoQualityBounds {
        AutoQualityBounds {
            max_scale: 1.0,
            min_scale: 0.5,
            preset_steps: steps,
            budget_ms: BUDGET,
            upscaler: None,
            fsr: false,
        }
    }

    /// A GPU whose time is `full` ms at scale 1 and the game's quality:
    /// the pixels scale it, each preset step takes a fifth off.
    fn gpu_cost(full: f64, q: &AutoQuality) -> f64 {
        let s = f64::from(q.scale());
        full * s * s * (1.0 - 0.2 * f64::from(q.step()))
    }

    /// The interval a vsync-locked display shows for a GPU cost: the next
    /// vsync after it.
    fn vsync(cost: f64) -> f64 {
        (cost / BUDGET).ceil().max(1.0) * BUDGET
    }

    fn run_gpu(q: &mut AutoQuality, full: f64, frames: u32) -> Vec<(f32, u8)> {
        let mut moves = Vec::new();
        for _ in 0..frames {
            let g = gpu_cost(full, q);
            if q.sample(FrameSample {
                interval_ms: vsync(g),
                cpu_ms: 3.0,
                gpu_ms: Some(g),
                upscale_ms: None,
            }) {
                moves.push((q.scale(), q.step()));
            }
        }
        moves
    }

    #[test]
    fn the_ladder_runs_from_the_most_to_the_least_scale_in_tenths() {
        assert_eq!(ladder(1.0, 0.5), vec![1.0, 0.9, 0.8, 0.7, 0.6, 0.5]);
        assert_eq!(
            ladder(1.25, 0.55),
            vec![1.25, 1.2, 1.1, 1.0, 0.9, 0.8, 0.7, 0.6, 0.55]
        );
        assert_eq!(ladder(0.75, 0.75), vec![0.75]);
        // Out of bounds, and a minimum above the maximum.
        assert_eq!(ladder(5.0, 0.1)[0], 2.0);
        assert_eq!(*ladder(5.0, 0.1).last().unwrap(), 0.5);
        assert_eq!(ladder(0.6, 0.9), vec![0.6]);
    }

    /// A GPU at 30 ms for 16.7: the scale goes down a tenth at a time; it
    /// stops at the first rung under 0.9 of the budget (30 × 0.8² = 19.2 ms
    /// is past 15, 30 × 0.7² = 14.7 is not) and stays there, though scale
    /// 0.8 is not tried again (19.2 is past 0.75 of the budget).
    #[test]
    fn an_overloaded_gpu_lowers_the_scale_first_and_settles() {
        let mut q = AutoQuality::new(bounds(2));
        let moves = run_gpu(&mut q, 30.0, 3000);
        assert_eq!(moves, vec![(0.9, 0), (0.8, 0), (0.7, 0)]);
        // Far too slow: every rung down, presets last, and no further.
        let mut q = AutoQuality::new(bounds(2));
        let moves = run_gpu(&mut q, 200.0, 5000);
        assert_eq!(moves.last(), Some(&(0.5, 2)));
        assert_eq!(moves.len(), 7);
        assert!(moves[..5].iter().all(|m| m.1 == 0), "{moves:?}");
    }

    /// Between the thresholds nothing moves: 0.8 of the budget at scale
    /// 0.9 is under DOWN_AT, and scale 1 would be 0.99 of it, over UP_AT.
    #[test]
    fn a_cost_between_the_thresholds_never_moves() {
        let mut q = AutoQuality::new(bounds(2));
        q.rung = 1;
        let full = 0.8 * BUDGET / (0.9 * 0.9);
        assert_eq!(run_gpu(&mut q, full, 10_000), vec![]);
        assert_eq!(q.scale(), 0.9);
    }

    /// Room comes back (a lighter scene): the presets come back first, then
    /// the scale, one rung at a time up to the game's scale.
    #[test]
    fn room_brings_the_presets_back_before_the_scale() {
        let mut q = AutoQuality::new(bounds(2));
        run_gpu(&mut q, 200.0, 5000);
        assert_eq!((q.scale(), q.step()), (0.5, 2));
        let moves = run_gpu(&mut q, 6.0, 5000);
        assert_eq!(
            moves,
            vec![
                (0.5, 1),
                (0.5, 0),
                (0.6, 0),
                (0.7, 0),
                (0.8, 0),
                (0.9, 0),
                (1.0, 0)
            ]
        );
    }

    /// One hitch (a pipeline compiled, a level streamed in) is not a trend.
    #[test]
    fn a_hitch_moves_nothing() {
        let mut q = AutoQuality::new(bounds(2));
        for i in 0..2000 {
            let late = i % 200 == 100;
            q.sample(FrameSample {
                interval_ms: if late { 60.0 } else { BUDGET },
                cpu_ms: 3.0,
                gpu_ms: Some(if late { 50.0 } else { 8.0 }),
                upscale_ms: None,
            });
        }
        assert_eq!((q.scale(), q.step()), (1.0, 0));
    }

    /// Without GPU time: a GPU that misses every vsync at scale 1 but not
    /// at 0.9 goes down, then tries 1 again after a calm while, fails, and
    /// waits twice as long each time; after MAX_FAILURES it stays at 0.9.
    /// Ten minutes of play see at most that many reversals.
    #[test]
    fn trials_up_without_gpu_time_back_off_and_stop() {
        let mut q = AutoQuality::new(bounds(2));
        let mut moves = Vec::new();
        for _ in 0..36_000 {
            let interval = vsync(gpu_cost(18.0, &q));
            if q.sample(FrameSample {
                interval_ms: interval,
                cpu_ms: 3.0,
                gpu_ms: None,
                upscale_ms: None,
            }) {
                moves.push(q.scale());
            }
        }
        let ups = moves.iter().filter(|s| **s == 1.0).count();
        assert_eq!(ups, MAX_FAILURES as usize, "{moves:?}");
        assert_eq!(moves.len(), 1 + 2 * MAX_FAILURES as usize, "{moves:?}");
        assert_eq!(q.scale(), 0.9);
    }

    /// A deterministic noise source in -1..1 (an LCG): the frames' jitter.
    struct Noise(u64);

    impl Noise {
        fn next(&mut self) -> f64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (self.0 >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
        }
    }

    /// studio-win's cycle (G1-4 on Chrome and Edge, D3D12, meridian-loop at
    /// 2560×1600, DPR 2, the high preset, renderScale 2, a ~56 Hz panel):
    /// at 1.6 the rung above, 1.7, was predicted just under the up line
    /// (0.75 × 17.9 = 13.4 ms) but measured near 14.6 ms, and a content spike
    /// every ~25 s took it over the down line (16.1 ms) and back to 1.6,
    /// which then stepped up again two seconds later: one reversal per
    /// spike for as long as the game ran. Scripted here: the GPU's time per
    /// rung with ±0.6 ms of jitter and a 4 ms spike of 40 frames every
    /// 1500. Before the fix it went 1.6 → 1.7 → 1.6 every spike; now the
    /// spread keeps 1.7 from being tried at all, and nothing moves.
    #[test]
    fn a_noisy_rung_predicted_at_the_up_line_is_not_cycled() {
        let budget = 17.9;
        let mut q = AutoQuality::new(AutoQualityBounds {
            max_scale: 2.0,
            min_scale: 0.5,
            preset_steps: 1,
            budget_ms: budget,
            upscaler: None,
            fsr: false,
        });
        // Measured, not quadratic: 1.6's 11.7 predicts 13.2 for 1.7, which measures 14.6.
        let base = |s: f32| match (s * 10.0).round() as i32 {
            20 => 24.0,
            19 => 21.0,
            18 => 18.0,
            17 => 14.6,
            16 => 11.7,
            k => 11.7 * f64::from(k) * f64::from(k) / 256.0,
        };
        let mut noise = Noise(7);
        let mut moves = Vec::new();
        for i in 0..36_000u32 {
            let spike = if i % 1500 < 40 && i > 3000 { 4.0 } else { 0.0 };
            let g = base(q.scale()) + 0.6 * noise.next() + spike;
            if q.sample(FrameSample {
                interval_ms: (g / budget).ceil().max(1.0) * budget,
                cpu_ms: 3.0,
                gpu_ms: Some(g),
                upscale_ms: None,
            }) {
                moves.push(q.scale());
            }
        }
        // Down from 2.0 to 1.6 at the start, then held: no reversals.
        assert_eq!(moves, vec![1.9, 1.8, 1.7, 1.6], "{moves:?}");
        assert_eq!(q.scale(), 1.6);
    }

    /// With GPU time, a rung that measures over the down line as soon as
    /// it is tried (predicted well under, noise-free, so the spread adds
    /// nothing) fails its trial: the next try waits twice as long, and
    /// after MAX_FAILURES it is not tried again. Before the fix it went up
    /// and down every UP_FRAMES + DOWN_FRAMES for as long as the game ran.
    #[test]
    fn a_step_up_that_falls_back_with_gpu_time_backs_off_and_stops() {
        let mut q = AutoQuality::new(bounds(0));
        let mut moves = Vec::new();
        for _ in 0..36_000 {
            // 0.9 costs 8 ms (predicting 9.9 for 1.0), 1.0 costs 17.
            let g = if q.scale() >= 1.0 { 17.0 } else { 8.0 };
            if q.sample(FrameSample {
                interval_ms: (g / BUDGET).ceil().max(1.0) * BUDGET,
                cpu_ms: 3.0,
                gpu_ms: Some(g),
                upscale_ms: None,
            }) {
                moves.push(q.scale());
            }
        }
        let ups = moves.iter().filter(|s| **s == 1.0).count();
        assert_eq!(ups, MAX_FAILURES as usize, "{moves:?}");
        assert_eq!(moves.len(), 1 + 2 * MAX_FAILURES as usize, "{moves:?}");
        assert_eq!(q.scale(), 0.9);
    }

    /// A step down that saves much less than its pixels predict goes back
    /// to the last rung that helped and stays. The Last Light on Chrome
    /// WebGPU (2560×1440, no GPU time, 60 Hz), its median interval per
    /// rung with ±8 ms of jitter: 0.9 and 0.8 saved 19 and 18 ms, 0.7 cost
    /// 3 ms more. Before the fix it went on to 0.5 and a preset step,
    /// slower than at 0.8. With GPU time, a GPU whose time the pixels set
    /// but the preset step does not change takes that step and comes back.
    #[test]
    fn a_step_down_that_does_not_save_its_pixels_comes_back_and_stays() {
        // The preset step at 0.5 measured 100.2.
        let interval = |q: &AutoQuality| match (q.scale() * 10.0).round() as i32 {
            10 => 119.5,
            9 => 100.6,
            8 => 82.4,
            7 => 85.6,
            6 => 91.9,
            _ => 100.4,
        };
        let mut q = AutoQuality::new(bounds(1));
        let mut noise = Noise(3);
        let mut moves = Vec::new();
        for _ in 0..36_000 {
            if q.sample(FrameSample {
                interval_ms: interval(&q) + 8.0 * noise.next(),
                cpu_ms: 5.0,
                gpu_ms: None,
                upscale_ms: None,
            }) {
                moves.push((q.scale(), q.step()));
            }
        }
        assert_eq!(moves, vec![(0.9, 0), (0.8, 0), (0.7, 0), (0.8, 0)]);

        let mut q = AutoQuality::new(bounds(2));
        let mut moves = Vec::new();
        for _ in 0..36_000 {
            let s = f64::from(q.scale());
            let g = 80.0 * s * s;
            if q.sample(FrameSample {
                interval_ms: vsync(g),
                cpu_ms: 3.0,
                gpu_ms: Some(g),
                upscale_ms: None,
            }) {
                moves.push((q.scale(), q.step()));
            }
        }
        assert_eq!(
            moves,
            vec![
                (0.9, 0),
                (0.8, 0),
                (0.7, 0),
                (0.6, 0),
                (0.5, 0),
                (0.5, 1),
                (0.5, 0)
            ]
        );
    }

    /// Without GPU time, late frames the CPU's own work explains (a heavy
    /// simulation) lower nothing: fewer pixels would not help.
    #[test]
    fn a_cpu_bound_game_keeps_its_scale() {
        let mut q = AutoQuality::new(bounds(2));
        for _ in 0..3000 {
            q.sample(FrameSample {
                interval_ms: 2.0 * BUDGET,
                cpu_ms: 25.0,
                gpu_ms: None,
                upscale_ms: None,
            });
        }
        assert_eq!((q.scale(), q.step()), (1.0, 0));
    }

    /// A GPU whose frame is `pixels` ms at scale 1 (by the scale's area)
    /// plus its upscaler's resolve, TAAU's `taau` ms and FSR's
    /// [`FSR_PER_TAAU`] times that, under `auto` with FSR: the moves.
    fn run_upscaled(pixels: f64, taau: f64, frames: u32) -> Vec<(f32, u8)> {
        let mut q = AutoQuality::new(AutoQualityBounds {
            upscaler: Some(Upscaler::Auto),
            fsr: true,
            ..bounds(0)
        });
        let mut moves = Vec::new();
        for _ in 0..frames {
            let s = f64::from(q.scale());
            let resolve = match q.rung().upscaler {
                Some(Upscaler::Fsr3) => taau * FSR_PER_TAAU,
                _ => taau,
            };
            let g = pixels * s * s + resolve;
            if q.sample(FrameSample {
                interval_ms: vsync(g),
                cpu_ms: 3.0,
                gpu_ms: Some(g),
                upscale_ms: Some(resolve),
            }) {
                moves.push((q.scale(), q.step()));
            }
        }
        moves
    }

    /// `auto` draws TAAU at scale 1 and FSR below it, whose resolve costs
    /// more: 1.0 → 0.9 saves less than its pixels. Priced by the pixels
    /// alone, that step was judged not to help, went back to 1.0 and held
    /// there over the budget (the half-systems audit's #18). With each
    /// rung's upscaler priced it goes on down to the rung that fits.
    #[test]
    fn a_step_into_fsr_is_priced_with_its_resolve() {
        let (pixels, taau) = (18.0, 0.8);
        let at = |s: f64, fsr: bool| pixels * s * s + if fsr { taau * FSR_PER_TAAU } else { taau };
        // The scenario: past the budget at 1.0 and 0.9, under it at 0.8,
        // and 0.9 saves less than half of what its pixels predict.
        assert!(at(1.0, false) > DOWN_AT * BUDGET && at(0.9, true) > DOWN_AT * BUDGET);
        assert!(at(0.8, true) < DOWN_AT * BUDGET);
        assert!(at(1.0, false) - at(0.9, true) < HELP_AT * at(1.0, false) * (1.0 - 0.81));
        assert_eq!(run_upscaled(pixels, taau, 20_000), vec![(0.9, 0), (0.8, 0)]);
    }

    /// A rung that costs more than the one above it (0.9 under FSR against
    /// 1.0 under TAAU, where the resolve is much of the frame) is passed
    /// over on the way down and on the way back up.
    #[test]
    fn a_rung_dearer_than_the_one_above_is_passed_over() {
        let (pixels, taau) = (14.0, 1.5);
        let at = |s: f64, fsr: bool| pixels * s * s + if fsr { taau * FSR_PER_TAAU } else { taau };
        assert!(at(0.9, true) >= at(1.0, false), "0.9 is dominated");
        assert!(at(1.0, false) > DOWN_AT * BUDGET);
        let moves = run_upscaled(pixels, taau, 20_000);
        assert!(!moves.contains(&(0.9, 0)), "{moves:?}");
        assert_eq!(moves.first(), Some(&(0.8, 0)), "{moves:?}");
    }

    /// The bounds hold: no preset step when none is offered, never under the
    /// least scale; and new bounds keep the rung's scale.
    #[test]
    fn the_bounds_hold_and_new_bounds_keep_the_scale() {
        let mut q = AutoQuality::new(AutoQualityBounds {
            min_scale: 0.7,
            ..bounds(0)
        });
        run_gpu(&mut q, 500.0, 5000);
        assert_eq!((q.scale(), q.step()), (0.7, 0));
        q.set_bounds(bounds(2));
        assert_eq!((q.scale(), q.step()), (0.7, 0));
        q.set_bounds(AutoQualityBounds {
            max_scale: 0.6,
            ..bounds(2)
        });
        assert_eq!(q.scale(), 0.6);
    }
}
