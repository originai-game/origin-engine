//! The wall clock the host tells the engine, moved on by the ticks since.

use super::*;

/// The fixed tick's length in milliseconds of the engine's world.
pub(super) fn tick_ms(world: u32) -> f64 {
    engine(world).map_or(1000.0 / 60.0, |e| {
        1000.0 / f64::from(e.state.borrow().rate.max(1))
    })
}

/// Moves the clock's base to a time the host gave at `tick`. The same
/// clock never goes back: a time older than the base moved on by the ticks
/// since (a harness that advanced faster than the host's clock, or answers
/// stamped by the Player and by the browser in turn) keeps the moved-on
/// base. A different clock (`clock` names another kind) starts afresh.
pub(super) fn rebase(p: &mut Platform, time: f64, tick: u64, clock: Option<&str>, tick_ms: f64) {
    let switched = clock.is_some_and(|c| p.clock.as_deref() != Some(c));
    let time = match p.base {
        Some((t, at)) if !switched => time.max(t + tick.saturating_sub(at) as f64 * tick_ms),
        _ => time,
    };
    p.base = Some((time, tick));
}

/// og.Time jumped from tick `from` to `to` (a game restored a save): the
/// clock keeps the time it had, moving on from `to`.
pub(crate) fn shift_ticks(engine: u32, from: u64, to: u64) {
    if from == to {
        return;
    }
    let ms = tick_ms(engine);
    with(engine, |p| {
        if let Some((time, at)) = p.base {
            p.base = Some((time + from.saturating_sub(at) as f64 * ms, to));
        }
    });
}

pub fn og_platform_now(world: u32) -> f64 {
    let Some((time, at)) = with(world, |p| p.base) else {
        return 0.0;
    };
    let tick = tick_now(world).unwrap_or(at);
    time + tick.saturating_sub(at) as f64 * tick_ms(world)
}
