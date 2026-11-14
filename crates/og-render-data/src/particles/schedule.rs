use super::*;

// ---------------------------------------------------------------------------
// The spawn schedule (computed on the CPU for both paths).
// ---------------------------------------------------------------------------

/// What one tick of a system does.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TickStep {
    pub tick: u64,
    /// Seconds since the system started, at the end of this tick.
    pub time: f32,
    /// The index of the first particle spawned this tick.
    pub first: u64,
    /// Particles spawned: bursts first (age offset 0), then continuous ones
    /// spread over the tick.
    pub count: u32,
    pub bursts: u32,
    /// The emitter's transform during this tick.
    pub transform: Mat4,
    /// The seconds of particle time this tick takes: the tick's length
    /// times the system's time scale.
    pub dt: f32,
    /// The emitter's velocity during this tick (its translation's move
    /// since the last frame over the ticks between; 0 after a reset).
    pub velocity: Vec3,
}

/// How many bursts (summed counts) have fired strictly before `end` seconds
/// into one cycle.
fn bursts_in_cycle(bursts: &[Burst], end: f64, duration: f64) -> u64 {
    let mut n = 0u64;
    for b in bursts {
        let cycles = b.cycles.max(1) as u64;
        let interval = b.interval.max(0.0) as f64;
        for i in 0..cycles {
            let t = b.time.max(0.0) as f64 + i as f64 * interval;
            if t >= duration && duration > 0.0 || t >= end {
                break;
            }
            n += b.count as u64;
        }
    }
    n
}

/// Particles spawned with a spawn time before `elapsed` seconds:
/// (continuous, bursts).
pub(super) fn spawned_before(e: &ParticleEmitter, elapsed: f64, stop: f64) -> (u64, u64) {
    let duration = (e.duration.max(0.0) as f64).max(1e-6);
    let end = if e.looping {
        elapsed.min(stop)
    } else {
        elapsed.min(stop).min(duration)
    };
    if end <= 0.0 {
        return (0, 0);
    }
    let continuous = match &e.rate_over_time {
        None => (e.rate.max(0.0) as f64 * end).floor() as u64,
        // The rate's integral over whole cycles and the part of the last.
        Some(c) => {
            let cycle = |t: f64| f64::from(c.integral((t / duration) as f32).max(0.0)) * duration;
            let full = (end / duration).floor();
            let area = full * cycle(duration) + cycle(end - full * duration);
            (e.rate.max(0.0) as f64 * area).floor() as u64
        }
    };
    let bursts = if e.looping {
        let full = (end / duration).floor();
        let per_cycle = bursts_in_cycle(&e.bursts, duration, duration);
        full as u64 * per_cycle + bursts_in_cycle(&e.bursts, end - full * duration, duration)
    } else {
        bursts_in_cycle(&e.bursts, end, duration)
    };
    (continuous, bursts)
}

/// The spawns of `tick` for a system.
pub fn tick_step(
    e: &ParticleEmitter,
    sys: &ParticleSystem,
    tick: u64,
    tick_rate: f32,
    transform: Mat4,
    clock: &ParticleClock,
) -> TickStep {
    let dt = 1.0 / tick_rate.max(1e-3) as f64;
    let stop = sys
        .stop_tick
        .map_or(f64::INFINITY, |s| clock.at(s.max(sys.start_tick), dt));
    let (time, first, count, bursts) = if tick < sys.start_tick {
        (0.0, 0, 0, 0)
    } else {
        let (t0, t1) = (clock.at(tick, dt), clock.at(tick + 1, dt));
        let (c0, b0) = spawned_before(e, t0, stop);
        let (c1, b1) = spawned_before(e, t1, stop);
        let bursts = b1 - b0;
        (
            t1 as f32,
            c0 + b0,
            (c1 - c0 + bursts).min(u32::MAX as u64) as u32,
            bursts.min(u32::MAX as u64) as u32,
        )
    };
    TickStep {
        tick,
        time,
        first,
        count,
        bursts,
        transform,
        dt: (1.0 / tick_rate.max(1e-3)) * clock.scale,
        velocity: Vec3::ZERO,
    }
}

/// The ticks a system's state must step through to reach `sys.tick`, and
/// whether it restarts from empty.
///
/// With `history` holding the tick before `sys.tick`, every tick before
/// `sys.tick` steps as recorded (its transform, start, stop, scale and
/// clock), and `sys.tick` from the clock the tick before left; a restart
/// replays the window from its first recorded tick. Without, the frame's
/// system stands for them: `clock` is re-anchored at the system's time
/// scale, from its start on a reset (as if the scale had always held), else
/// from the first tick not yet stepped, and a moving emitter spawns along
/// the path between the frames.
#[allow(clippy::too_many_arguments)]
pub fn plan_ticks(
    e: &ParticleEmitter,
    sys: &ParticleSystem,
    last_tick: Option<u64>,
    prev_transform: Option<Mat4>,
    tick_rate: f32,
    clock: &mut ParticleClock,
    history: Option<&ParticleHistory>,
) -> (bool, Vec<TickStep>) {
    let scale = time_scale(sys);
    let window = window(e, scale, tick_rate);
    let dt = 1.0 / tick_rate.max(1e-3) as f64;
    let before = sys.tick.checked_sub(1);
    if let Some((h, prev)) = history.zip(before).and_then(|(h, b)| Some((h, h.at(b)?))) {
        let first = h.ticks.front().map_or(sys.tick, |t| t.tick);
        let (reset, from) = match last_tick {
            Some(last) if sys.tick >= last && sys.tick - last <= window && last + 1 >= first => {
                (false, last + 1)
            }
            _ => (true, sys.tick.saturating_sub(window - 1).max(first)),
        };
        *clock = next_clock(prev.clock, sys.tick, scale, dt);
        let now = *clock;
        let steps = (from..=sys.tick)
            .map(|t| match h.at(t).filter(|_| t < sys.tick) {
                Some(r) => {
                    let at = ParticleSystem {
                        transform: r.transform,
                        start_tick: r.start_tick,
                        stop_tick: r.stop_tick,
                        time_scale: r.time_scale,
                        tick: t,
                        ..*sys
                    };
                    tick_step(e, &at, t, tick_rate, r.transform, &r.clock)
                }
                None => tick_step(e, sys, t, tick_rate, sys.transform, &now),
            })
            .collect();
        return (reset, steps);
    }
    let (reset, from) = match last_tick {
        Some(last) if sys.tick >= last && sys.tick - last <= window => (false, last + 1),
        _ => (
            true,
            sys.tick.saturating_sub(window - 1).max(sys.start_tick),
        ),
    };
    if reset {
        *clock = ParticleClock {
            tick: sys.start_tick,
            time: 0.0,
            scale,
        };
    } else if clock.scale != scale {
        *clock = ParticleClock {
            tick: from,
            time: clock.at(from, dt),
            scale,
        };
    }
    let clock = *clock;
    let ticks: Vec<u64> = (from..=sys.tick).collect();
    let n = ticks.len();
    let velocity = match (reset, prev_transform) {
        (false, Some(prev)) if n > 0 => {
            (sys.transform.w_axis - prev.w_axis).truncate() * (tick_rate / n as f32)
        }
        _ => Vec3::ZERO,
    };
    let steps = ticks
        .into_iter()
        .enumerate()
        .map(|(i, t)| {
            // Moving emitters spawn along the path between the frames.
            let transform = match (reset, prev_transform) {
                (false, Some(prev)) if n > 1 => {
                    let f = (i + 1) as f32 / n as f32;
                    let mut m = sys.transform;
                    m.w_axis = prev.w_axis.lerp(sys.transform.w_axis, f);
                    m
                }
                _ => sys.transform,
            };
            TickStep {
                velocity,
                ..tick_step(e, sys, t, tick_rate, transform, &clock)
            }
        })
        .collect();
    (reset, steps)
}
