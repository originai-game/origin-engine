//! Simulation time: a plain value the world advances once per fixed tick.
//!
//! Game code never reads the wall clock; it reads `Time`. `elapsed` is
//! computed from `tick` (never accumulated), so it carries no drift and is
//! identical on every host.

use crate::Reflect;

/// The fixed-tick clock: which tick is running, its length, and the time at
/// its start.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Time")]
pub struct Time {
    /// Ticks completed before the current one (0 during the first tick).
    pub tick: u64,
    /// Seconds per tick.
    #[reflect(unit = "s")]
    pub dt: f32,
    /// Seconds of simulation before the current tick: `tick * dt`.
    #[reflect(unit = "s")]
    pub elapsed: f64,
}

impl Default for Time {
    /// 60 Hz, tick 0.
    fn default() -> Self {
        Time::from_rate(60)
    }
}

impl Time {
    /// A clock at `rate` ticks per second, at tick 0. A rate of 0 is taken as 1.
    pub fn from_rate(rate: u32) -> Self {
        Time {
            tick: 0,
            dt: 1.0 / rate.max(1) as f32,
            elapsed: 0.0,
        }
    }

    /// Moves to the next tick.
    pub fn advance(&mut self) {
        self.tick += 1;
        self.elapsed = self.tick as f64 * f64::from(self.dt);
    }
}

/// Global slow motion and hit stop: how much simulation time the modules
/// that honour it (physics, animation, flipbooks, tweens) advance per tick.
///
/// The tick itself never changes: every tick still runs every system, so the
/// UI, input and the probe keep their pace. A module reads [`TimeScale::dt`]
/// instead of [`Time::dt`]. During a hit stop (`hit_stop` ticks left) that is
/// 0; otherwise `Time::dt * scale`. The world counts `hit_stop` down and adds
/// the tick's scaled `dt` to `elapsed` after the tick's last system, so a hit
/// stop of `n` started during tick N freezes ticks N to N + n - 1.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.TimeScale")]
pub struct TimeScale {
    /// The rate of scaled time: 1 is normal, 0.5 half speed, 0 stopped.
    #[reflect(default = 1, min = 0, max = 10)]
    pub scale: f32,
    /// Ticks of hit stop left: while above 0, scaled time does not advance.
    #[reflect(name = "hitStop", unit = "ticks")]
    pub hit_stop: u32,
    /// Scaled seconds since load: the sum of every finished tick's scaled `dt`.
    #[reflect(unit = "s")]
    pub elapsed: f64,
}

/// The largest [`TimeScale::scale`].
pub const MAX_TIME_SCALE: f32 = 10.0;

impl Default for TimeScale {
    fn default() -> Self {
        TimeScale {
            scale: 1.0,
            hit_stop: 0,
            elapsed: 0.0,
        }
    }
}

impl TimeScale {
    /// The scaled length of the tick: 0 during a hit stop, else `time.dt * scale`.
    pub fn dt(&self, time: &Time) -> f32 {
        if self.hit_stop > 0 {
            0.0
        } else {
            time.dt * self.scale
        }
    }

    /// After the tick's last system: adds its scaled `dt`, counts the hit stop down.
    pub fn end_tick(&mut self, time: &Time) {
        self.elapsed += f64::from(self.dt(time));
        self.hit_stop = self.hit_stop.saturating_sub(1);
    }
}
