//! Automatic quality: from the measured frame time it moves the render
//! scale first, then the graphics preset, down when frames run over their
//! budget and back up when there is room, with hysteresis so that it never
//! oscillates. Pure logic: the host feeds it one [`FrameSample`] per drawn
//! frame and applies the [`AutoQuality::scale`] and [`AutoQuality::step`]
//! it answers (og-runtime's `display.rs`); it reads no clock and draws
//! nothing.
//!
//! The ladder is one list of rungs, best first: the render scale from the
//! most to the least in steps of [`SCALE_STEP`], then, at the least scale,
//! each preset step the host offers (a cap at medium, then at low). A step
//! down is one rung down and a step up one rung up, so the scale goes first
//! on the way down and comes back last on the way up.
//!
//! Hysteresis, two ways:
//!
//! - With the GPU's own time ([`FrameSample::gpu_ms`], timestamp queries),
//!   it steps down when the smoothed time passes [`DOWN_AT`] of the budget
//!   for [`DOWN_FRAMES`] frames in a row, and up only when the time the
//!   rung above would take (the scale's pixels, or [`PRESET_COST`] more for
//!   a preset step) stays under [`UP_AT`] for [`UP_FRAMES`] frames.
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

/// The render scale's step on the ladder.
pub const SCALE_STEP: f32 = 0.1;
/// Steps down past this fraction of the budget (GPU time).
pub const DOWN_AT: f64 = 0.9;
/// Steps up when the rung above would stay under this fraction (GPU time).
pub const UP_AT: f64 = 0.75;
/// A frame's interval past this fraction of the budget is late (no GPU time).
pub const LATE_AT: f64 = 1.2;
/// The cost a preset step adds back, as a fraction, when the GPU time
/// predicts a step up.
pub const PRESET_COST: f64 = 0.3;
/// Frames in a row over the budget before a step down.
pub const DOWN_FRAMES: u32 = 20;
/// Frames in a row with room before a step up (GPU time).
pub const UP_FRAMES: u32 = 120;
/// Calm frames before a trial step up (no GPU time), doubled per failure.
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
    /// The ladder's render scales, best first.
    scales: Vec<f32>,
    rung: usize,
    /// The smoothed cost (GPU ms, or the interval) and CPU time; `None`
    /// until the first sample after a change.
    cost: Option<f64>,
    cpu: f64,
    over: u32,
    calm: u32,
    settle: u32,
    trial: Option<Trial>,
    /// Failed trials per rung.
    failures: Vec<u32>,
}

impl AutoQuality {
    /// A controller at the top rung (the game's scale and quality).
    pub fn new(bounds: AutoQualityBounds) -> AutoQuality {
        let mut q = AutoQuality {
            bounds,
            scales: Vec::new(),
            rung: 0,
            cost: None,
            cpu: 0.0,
            over: 0,
            calm: 0,
            settle: 0,
            trial: None,
            failures: Vec::new(),
        };
        q.set_bounds(bounds);
        q
    }

    /// New bounds (settings or budget changed): the rung keeps its scale
    /// and step where they are still on the ladder (else the nearest), and
    /// every failed trial is forgotten.
    pub fn set_bounds(&mut self, bounds: AutoQualityBounds) {
        let (scale, step) = if self.scales.is_empty() {
            (f32::INFINITY, 0)
        } else {
            (self.scale(), self.step())
        };
        self.bounds = AutoQualityBounds {
            max_scale: finite_or(bounds.max_scale, 1.0),
            min_scale: finite_or(bounds.min_scale, 1.0),
            ..bounds
        };
        self.scales = ladder(self.bounds.max_scale, self.bounds.min_scale);
        let rungs = self.rungs();
        self.rung = if step > 0 {
            (self.scales.len() - 1 + usize::from(step)).min(rungs - 1)
        } else {
            self.scales
                .iter()
                .position(|s| *s <= scale + 1e-4)
                .unwrap_or(self.scales.len() - 1)
        };
        self.failures = vec![0; rungs];
        self.restart();
    }

    /// The bounds in force.
    pub fn bounds(&self) -> AutoQualityBounds {
        self.bounds
    }

    /// The render scale to draw at.
    pub fn scale(&self) -> f32 {
        self.scales[self.rung.min(self.scales.len() - 1)]
    }

    /// The preset steps taken below the game's quality (0: none).
    pub fn step(&self) -> u8 {
        (self.rung + 1).saturating_sub(self.scales.len()) as u8
    }

    /// The smoothed cost the decisions read, ms (GPU time, else the frame
    /// interval); 0 before a sample.
    pub fn cost_ms(&self) -> f64 {
        self.cost.unwrap_or(0.0)
    }

    fn rungs(&self) -> usize {
        self.scales.len() + usize::from(self.bounds.preset_steps)
    }

    fn restart(&mut self) {
        self.cost = None;
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
        let cost = gpu.unwrap_or(s.interval_ms);
        let smoothed = match self.cost {
            Some(c) => c * 0.9 + cost * 0.1,
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
            if self.over >= DOWN_FRAMES && self.rung + 1 < self.rungs() {
                if let Some(t) = self.trial.take()
                    && t.rung == self.rung
                {
                    self.failures[t.rung] += 1;
                }
                self.rung += 1;
                self.restart();
                return true;
            }
            return false;
        }
        self.over = 0;
        if self.rung == 0 {
            return false;
        }
        let up = self.rung - 1;
        let room = match gpu {
            Some(_) => self.predicted(up) < budget * UP_AT,
            None => smoothed <= budget * 1.05,
        };
        if !room {
            self.calm = 0;
            return false;
        }
        self.calm += 1;
        let wait = match gpu {
            Some(_) => UP_FRAMES,
            None => {
                if self.failures[up] >= MAX_FAILURES {
                    return false;
                }
                TRIAL_FRAMES << self.failures[up]
            }
        };
        if self.calm < wait {
            return false;
        }
        self.rung = up;
        self.trial = gpu.is_none().then_some(Trial {
            rung: up,
            frames_left: VERIFY_FRAMES + SETTLE_FRAMES,
        });
        self.restart();
        true
    }

    /// What the smoothed cost would be on rung `to`, one above this one.
    fn predicted(&self, to: usize) -> f64 {
        let cost = self.cost_ms();
        if to + 1 >= self.scales.len() {
            // A preset step back (or the step from the least scale's
            // first preset step back to the scale alone).
            return cost * (1.0 + PRESET_COST);
        }
        let ratio = f64::from(self.scales[to] / self.scales[to + 1]);
        cost * ratio * ratio
    }
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
            }) {
                moves.push(q.scale());
            }
        }
        let ups = moves.iter().filter(|s| **s == 1.0).count();
        assert_eq!(ups, MAX_FAILURES as usize, "{moves:?}");
        assert_eq!(moves.len(), 1 + 2 * MAX_FAILURES as usize, "{moves:?}");
        assert_eq!(q.scale(), 0.9);
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
            });
        }
        assert_eq!((q.scale(), q.step()), (1.0, 0));
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
