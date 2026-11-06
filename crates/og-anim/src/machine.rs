//! A graph layer's state machine, shared by every target a graph drives
//! (og.anim's skeletons, og.flipbook's sheets, og.skeleton2d's rigs): its
//! current and fading-out states, their phases, and the transition step.

use crate::asset::NONE;
use crate::graph::{Interrupt, Layer};
use crate::schema::GRAPH_PARAMS;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Phase {
    pub(crate) cycles: u32,
    pub(crate) frac: f32,
}

impl Phase {
    pub(crate) fn total(self) -> f64 {
        self.cycles as f64 + self.frac as f64
    }
    /// Advances by `d` cycles; returns the wraps.
    pub(crate) fn advance(&mut self, d: f32) -> i32 {
        let f = self.frac + d;
        let k = f.floor();
        self.frac = f - k;
        if self.frac >= 1.0 {
            self.frac = 0.0;
        }
        let k = k as i32;
        self.cycles = self.cycles.saturating_add_signed(k);
        k
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct LayerState {
    pub(crate) state: u32,
    pub(crate) phase: Phase,
    pub(crate) time: f32,
    pub(crate) prev: u32,
    pub(crate) prev_phase: Phase,
    pub(crate) fade: f32,
    pub(crate) fade_duration: f32,
    /// What may interrupt the fade in progress (the transition's
    /// `interrupt`); `Any` when none is.
    pub(crate) guard: Interrupt,
}

impl LayerState {
    /// A layer at its entry state.
    pub(crate) fn entry(layer: &Layer) -> LayerState {
        LayerState {
            state: layer.entry as u32,
            prev: NONE,
            ..LayerState::default()
        }
    }

    /// Fires the first of `layer`'s transitions (file order) that is open
    /// and whose conditions hold: the state it leaves fades out over the
    /// transition's `fade`. Returns whether one fired.
    pub(crate) fn transition(&mut self, layer: &Layer, params: &[f32; GRAPH_PARAMS]) -> bool {
        let ls = self;
        if ls.state as usize >= layer.states.len() {
            ls.state = layer.entry as u32;
        }
        // During a fade, the transition's `interrupt` says which may fire.
        let guard = if ls.prev != NONE {
            ls.guard
        } else {
            Interrupt::Any
        };
        for t in &layer.transitions {
            let (cur, prev) = (ls.state as usize, ls.prev as usize);
            // A source interrupt leaves from the state being faded out.
            let from_source = guard == Interrupt::Source;
            let open = match (guard, t.from) {
                (Interrupt::None, _) => false,
                (Interrupt::Source, Some(f)) => f == prev && t.to != cur,
                (Interrupt::Source, None) => false,
                (Interrupt::Next, Some(f)) => f == cur,
                (Interrupt::Next, None) => false,
                (Interrupt::Any, Some(f)) => f == cur,
                (Interrupt::Any, None) => t.to != cur,
            };
            if !open || !t.conditions.iter().all(|c| c.holds(params)) {
                continue;
            }
            let (left, left_phase) = if from_source {
                (prev, ls.prev_phase)
            } else {
                (cur, ls.phase)
            };
            if t.exit.is_some_and(|x| left_phase.total() < x as f64) {
                continue;
            }
            let carried = match (layer.states[left].sync, layer.states[t.to].sync) {
                // Between two states of one group the fraction carries over.
                (Some(a), Some(b)) if a == b => left_phase.frac,
                _ => 0.0,
            };
            // The fade continues from the state left (a source interrupt
            // drops the target it abandons).
            ls.prev = left as u32;
            ls.prev_phase = left_phase;
            ls.guard = t.interrupt;
            ls.state = t.to as u32;
            ls.phase = Phase {
                cycles: 0,
                frac: carried,
            };
            ls.time = 0.0;
            ls.fade = 0.0;
            ls.fade_duration = t.fade;
            return true;
        }
        false
    }

    /// Advances the cross-fade by `dt`: its progress after this tick, 1
    /// when no fade runs.
    pub(crate) fn fade_alpha(&mut self, dt: f32) -> f32 {
        if self.prev != NONE {
            self.fade += dt;
            if self.fade_duration > 0.0 && self.fade < self.fade_duration {
                self.fade / self.fade_duration
            } else {
                1.0
            }
        } else {
            1.0
        }
    }

    /// Ends the fade once its progress reached 1.
    pub(crate) fn settle(&mut self, alpha: f32) {
        if alpha >= 1.0 {
            self.prev = NONE;
            self.guard = Interrupt::Any;
        }
    }
}
