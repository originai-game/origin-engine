//! og.time: the time scale, hit stop and group scales, and the shared time a system reads.

use super::*;

/// The Time fields the time scale adds, for `og.Time` and the systems' `time`.
pub(crate) fn time_scale_json(scale: og_core::TimeScale, time: og_core::Time) -> Value {
    json!({
        "scale": scale.scale,
        "hitStopTicks": scale.hit_stop,
        "scaledDt": scale.seconds(&time),
        "scaledElapsed": scale.elapsed,
    })
}

/// A system call's time scale, written to the shared buffer as four
/// doubles: scale, hitStopTicks, scaledDt, scaledElapsed
/// ([`crate::host::shared`]; `dispatch` in runtime.ts reads them).
pub(crate) fn put_time(
    shared: &mut [u8],
    read: &TimeRead,
    scale: f32,
    hit_stop: u32,
    dt: f64,
    elapsed: f64,
) {
    for (i, v) in time_numbers(read, scale, hit_stop, dt, elapsed)
        .into_iter()
        .enumerate()
    {
        crate::host::shared::put_f64(shared, i, v);
    }
}

/// [`put_time`]'s numbers. The scale, an f32 field, is what JavaScript's
/// `Number` reads from its `{}` text: the double nearest its own shortest
/// digits (0.3, not the 0.30000001192092896 it widens to). The scaled step
/// and elapsed time are f64 clocks (`og_core::TimeScale::seconds`), passed
/// as they are; what is not finite is NaN.
pub(super) fn time_numbers(
    read: &TimeRead,
    scale: f32,
    hit_stop: u32,
    dt: f64,
    elapsed: f64,
) -> [f64; 4] {
    let finite = |x: f64| if x.is_finite() { x } else { f64::NAN };
    [
        read.f32_as_read(scale),
        f64::from(hit_stop),
        finite(dt),
        finite(elapsed),
    ]
}

/// An engine's last time scale as `Number` reads it
/// ([`TimeRead::f32_as_read`]): it rarely changes, so it is kept.
#[derive(Default)]
pub(crate) struct TimeRead(std::cell::Cell<(u32, f64)>);

impl TimeRead {
    /// The double `Number` reads from an f32's `{}` text, the last answer
    /// kept.
    pub(super) fn f32_as_read(&self, x: f32) -> f64 {
        let mut last = self.0.get();
        if last.0 != x.to_bits() {
            // Core's `{}` rather than zmij: on an exact tie between two
            // shortest candidates zmij rounds to even and core up.
            let read = if x.is_finite() {
                x.to_string().parse().unwrap_or(f64::NAN)
            } else {
                f64::NAN
            };
            last = (x.to_bits(), read);
            self.0.set(last);
        }
        last.1
    }
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
