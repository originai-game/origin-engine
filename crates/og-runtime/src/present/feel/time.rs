//! og.time: the time scale, hit stop and group scales, and the shared time a system reads.

use super::*;

/// The Time fields the time scale adds, for `og.Time` and the systems' `time`.
pub(crate) fn time_scale_json(scale: og_core::TimeScale, time: og_core::Time) -> Value {
    json!({
        "scale": scale.scale,
        "hitStop": scale.hit_stop,
        "scaledDt": scale.dt(&time),
        "scaledElapsed": scale.elapsed,
    })
}

/// A system call's time scale, written to the shared buffer as four
/// doubles: scale, hitStop, scaledDt, scaledElapsed
/// ([`crate::host::shared`]; `dispatch` in runtime.ts reads them).
pub(crate) fn put_time(shared: &mut [u8], scale: f32, hit_stop: u32, dt: f32, elapsed: f64) {
    for (i, v) in time_numbers(scale, hit_stop, dt, elapsed)
        .into_iter()
        .enumerate()
    {
        crate::host::shared::put_f64(shared, i, v);
    }
}

/// [`put_time`]'s numbers: what JavaScript's `Number` reads from `{}`'s
/// text of each, which is what a system's `time` has always held (the
/// call once carried that text). An f32 is therefore the double nearest
/// its own shortest digits (0.016666668, not the 0.01666666753590107 it
/// widens to), and what `Number` does not read (`inf`) is NaN.
pub(super) fn time_numbers(scale: f32, hit_stop: u32, dt: f32, elapsed: f64) -> [f64; 4] {
    [
        f32_as_read(scale, 0),
        f64::from(hit_stop),
        f32_as_read(dt, 1),
        if elapsed.is_finite() {
            elapsed
        } else {
            f64::NAN
        },
    ]
}

/// The double `Number` reads from an f32's `{}` text. The scale and the
/// scaled step rarely change, so the last answer of each (`slot`) is kept.
pub(super) fn f32_as_read(x: f32, slot: usize) -> f64 {
    thread_local! {
        static LAST: std::cell::Cell<[(u32, f64); 2]> =
            const { std::cell::Cell::new([(0, 0.0); 2]) };
    }
    let mut last = LAST.with(std::cell::Cell::get);
    if last[slot].0 != x.to_bits() {
        // Core's `{}` rather than zmij: on an exact tie between two
        // shortest candidates zmij rounds to even and core up.
        let read = if x.is_finite() {
            x.to_string().parse().unwrap_or(f64::NAN)
        } else {
            f64::NAN
        };
        last[slot] = (x.to_bits(), read);
        LAST.with(|l| l.set(last));
    }
    last[slot].1
}

pub fn og_time_set_scale(world: u32, scale: f64) -> Result<(), String> {
    engine(world).and_then(|eng| {
        let s = scale as f32;
        with_world(&eng, |c| c.set_time_scale(s), |w| w.set_time_scale(s))
    })
}

pub fn og_time_hit_stop(world: u32, ticks: u32) -> Result<(), String> {
    engine(world).and_then(|eng| with_world(&eng, |c| c.hit_stop(ticks), |w| w.hit_stop(ticks)))
}

pub fn og_time_set_group_scale(world: u32, group: u32, scale: f64) -> Result<(), String> {
    engine(world).and_then(|eng| {
        let s = scale as f32;
        let name = "og.TimeGroups";
        if running(eng.id).is_none() {
            crate::engine::lazy::ensure(&eng, eng.world()?, name)?;
        }
        with_world(
            &eng,
            |c| {
                c.set_group_scale(group, s).map_err(|e| {
                    if e.kind() == og_core::ErrorKind::NotFound {
                        og_core::Error::new(
                            og_core::ErrorKind::NotFound,
                            format!(
                                "{name} is not registered yet: inside a system, name og.TimeGroup or {name} in a system's reads or writes, or call og.time.setGroupScale once in setup"
                            ),
                        )
                    } else {
                        e
                    }
                })
            },
            |w| w.set_group_scale(group, s),
        )
    })
}

pub fn og_time_group_dt(world: u32, entity: f64) -> f64 {
    let e = Entity::from_bits(entity as u64);
    engine(world)
        .and_then(|eng| {
            with_world(
                &eng,
                |c| Ok(c.group_dt(c.time_groups().as_ref(), e, c.scaled_dt())),
                |w| {
                    let dt = w.scaled_dt();
                    Ok(w.time_groups().map_or(dt, |g| dt * w.group_scale(&g, e)))
                },
            )
        })
        .map_or(0.0, f64::from)
}
