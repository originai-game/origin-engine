//! A damped spring stepped by the tick: the motion tokens' springs
//! (`motion.spring.*`: a damping ratio and a response in milliseconds, as
//! SwiftUI's `.spring(response:dampingFraction:)` and the web runtime's
//! spring) integrated with arithmetic only, so a value driven by it is the
//! same bits on every host and replays after a snapshot.
//!
//! The spring pulls `value` towards `target` with stiffness `(2π/response)²`
//! and damping `2·ζ·(2π/response)` per unit mass; each tick is four
//! semi-implicit Euler substeps. A spring within `1e-3` of its target and
//! slower than `1e-2` per second snaps to rest there.

/// A spring's character: its damping ratio (1 settles without overshoot,
/// below 1 bounces) and its response, the period of the undamped motion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    pub damping: f32,
    pub response_ms: f32,
}

impl Default for Spring {
    /// `motion.spring.default` of the design tokens (damping 1, 350 ms).
    fn default() -> Spring {
        Spring {
            damping: 1.0,
            response_ms: 350.0,
        }
    }
}

/// Where a spring is and how fast it moves (units per second).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SpringState {
    pub value: f32,
    pub velocity: f32,
}

const TAU: f32 = std::f32::consts::TAU;
const SUBSTEPS: u32 = 4;

impl Spring {
    /// One tick of `dt` seconds towards `target`.
    pub fn step(&self, s: SpringState, target: f32, dt: f32) -> SpringState {
        let response = (self.response_ms / 1000.0).max(1e-3);
        let omega = TAU / response;
        let k = omega * omega;
        let c = 2.0 * self.damping.max(0.0) * omega;
        let h = dt / SUBSTEPS as f32;
        let (mut x, mut v) = (s.value, s.velocity);
        for _ in 0..SUBSTEPS {
            v += (-k * (x - target) - c * v) * h;
            x += v * h;
        }
        if (x - target).abs() < 1e-3 && v.abs() < 1e-2 {
            return SpringState {
                value: target,
                velocity: 0.0,
            };
        }
        SpringState {
            value: x,
            velocity: v,
        }
    }

    /// Whether `s` rests at `target`.
    pub fn settled(s: SpringState, target: f32) -> bool {
        s.value == target && s.velocity == 0.0
    }
}
