//! Simulation time: a plain value the world advances once per fixed tick.
//!
//! Game code never reads the wall clock; it reads `Time`. `elapsed` is
//! computed from `tick` (never accumulated), so it carries no drift and is
//! identical on every host.

use crate::Reflect;

/// The fixed-tick clock: which tick is running, its length, and the time at
/// its start.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
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
