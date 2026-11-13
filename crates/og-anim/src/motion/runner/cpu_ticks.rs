//! The CPU model's wall time per tick, a diagnostic.

use serde_json::{Value, json};
use std::collections::VecDeque;

/// The CPU model's wall time per tick, on the engine thread (the lanes'
/// fork and join included): what one tick's share of the batches costs
/// each thread. A diagnostic; the last [`CPU_TICKS`] ticks with work.
#[derive(Default)]
pub struct CpuTicks {
    pub tick: Option<u64>,
    pub recent: VecDeque<f64>,
    pub ticks: u64,
    pub total: f64,
    pub max: f64,
}

/// How many ticks [`CpuTicks`] keeps for its percentiles.
pub const CPU_TICKS: usize = 4096;

impl CpuTicks {
    pub fn add(&mut self, tick: u64, ms: f64) {
        if self.tick == Some(tick)
            && let Some(last) = self.recent.back_mut()
        {
            *last += ms;
            self.max = self.max.max(*last);
        } else {
            if self.recent.len() == CPU_TICKS {
                self.recent.pop_front();
            }
            self.recent.push_back(ms);
            self.tick = Some(tick);
            self.ticks += 1;
            self.max = self.max.max(ms);
        }
        self.total += ms;
    }

    pub fn json(&self) -> Value {
        let mut sorted: Vec<f64> = self.recent.iter().copied().collect();
        sorted.sort_by(f64::total_cmp);
        let at = |q: f64| {
            sorted
                .get(((sorted.len() as f64 * q).ceil() as usize).saturating_sub(1))
                .copied()
                .unwrap_or(0.0)
        };
        json!({
            "ticks": self.ticks,
            "meanMs": if self.ticks > 0 { self.total / self.ticks as f64 } else { 0.0 },
            "p50Ms": at(0.5),
            "p95Ms": at(0.95),
            "maxMs": self.max,
            "threads": og_lanes::threads(),
        })
    }
}
