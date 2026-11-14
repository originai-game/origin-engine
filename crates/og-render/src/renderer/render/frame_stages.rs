//! The frame's CPU stages in order, each timed with the renderer's clock
//! ([`Renderer::stage_timings`]): where a frame's `render` call spends its
//! time before, during and after its graph's encoding. Measurement only:
//! nothing drawn depends on it, and without a clock it reads no time.

use super::*;

/// The stages a frame has finished, and when the current one began.
pub(super) struct Stages {
    clock: Option<fn() -> f64>,
    at: f64,
    pub done: Vec<PassTiming>,
}

impl Stages {
    /// The first stage begins now.
    pub fn start(clock: Option<fn() -> f64>) -> Stages {
        Stages {
            clock,
            at: clock.map_or(0.0, |c| c()),
            done: Vec::new(),
        }
    }

    /// Ends the current stage as `name`; the next begins now.
    pub fn end(&mut self, name: &str) {
        if let Some(c) = self.clock {
            let now = c();
            self.done.push(PassTiming {
                name: name.to_string(),
                start: self.at,
                ms: (now - self.at).max(0.0),
            });
            self.at = now;
        }
    }
}
