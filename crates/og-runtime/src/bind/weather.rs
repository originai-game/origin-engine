//! og.weather (the R4 round): og.WeatherState simulated in the tick from
//! og.Weather — wetness, puddles and lying snow, and lightning strikes at
//! random ticks from a stream forked from og.Rng and kept in the state, so
//! a replay, a restore and every host strike alike. og.WeatherState and
//! og.LightningStrike are lazy (crate::engine::lazy): until a game names one, this
//! system finds nothing to do and the world keeps its type table and hash.
//! Plain f32 arithmetic only (no transcendental functions), so every host
//! computes the same bits.

use crate::host::codec::Layout;
use crate::host::wire::{FromJson, record};
use glam::Vec3;
use og_ecs::{SchemaId, SystemContext, SystemDesc, World};
use serde_json::json;
use std::collections::BTreeMap;

pub(crate) const STATE: &str = "og.WeatherState";
pub(crate) const STRIKE: &str = "og.LightningStrike";
pub(crate) const SYSTEM: &str = "og.weather";
const WEATHER: &str = "og.Weather";
/// og.WeatherState and its event, registered together (crate::engine::lazy).
pub(crate) const RECORDS: [&str; 2] = [STATE, STRIKE];

/// The kinds og.Weather.kind names (og-render's WEATHER_*).
const RAIN: u64 = 1;
const SNOW: u64 = 2;
const DUST: u64 = 3;

/// The inputs a tick of the state reads (og.Weather and og.WeatherState's
/// own fields), the numbers it writes back.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Step {
    pub wetness: f32,
    pub puddles: f32,
    pub snow_depth: f32,
    pub dust: f32,
}

/// The rules: rain soaks the ground over soakTime and, once it is more than
/// half soaked, fills the puddles over puddleTime; without rain both dry
/// (dryTime, drainTime). Snow settles at snowfall per minute and melts at
/// melt per minute when none falls, three times as fast in rain; snow
/// keeps the ground under it from drying no faster than it would. A dust
/// storm settles dust over dustTime; it stays until rain washes it off
/// over washTime.
pub(crate) fn step(
    s: Step,
    state: &record::WeatherState,
    kind: u64,
    intensity: f32,
    dt: f32,
) -> Step {
    let rain = if kind == RAIN {
        intensity.clamp(0.0, 2.0)
    } else {
        0.0
    };
    let snowing = if kind == SNOW {
        intensity.clamp(0.0, 2.0)
    } else {
        0.0
    };
    let mut o = s;
    if rain > 0.0 {
        o.wetness += dt * rain / state.soak_time.max(0.1);
        if s.wetness > 0.5 {
            o.puddles += dt * rain / state.puddle_time.max(0.1);
        }
    } else {
        o.wetness -= dt / state.dry_time.max(0.1);
        o.puddles -= dt / state.drain_time.max(0.1);
    }
    if snowing > 0.0 {
        o.snow_depth += dt * snowing * state.snowfall.max(0.0) / 60.0;
    } else {
        o.snow_depth -= dt * state.melt.max(0.0) / 60.0 * (1.0 + 2.0 * rain);
    }
    if kind == DUST {
        o.dust += dt * intensity.clamp(0.0, 2.0) / state.dust_time.max(0.1);
    }
    o.dust -= dt * rain / state.wash_time.max(0.1);
    o.dust = o.dust.clamp(0.0, 1.0);
    o.wetness = o.wetness.clamp(0.0, 1.0);
    o.puddles = o.puddles.clamp(0.0, 1.0);
    o.snow_depth = o.snow_depth.max(0.0);
    o
}

/// A strike's offset in the unit disc: rejection sampling, no trig.
pub(crate) fn disc(rng: &mut og_core::Rng) -> (f32, f32) {
    loop {
        let x = rng.range_f32(-1.0, 1.0);
        let z = rng.range_f32(-1.0, 1.0);
        if x * x + z * z <= 1.0 {
            return (x, z);
        }
    }
}

/// Schedules og.weather (early in the tick, before the game's systems, so
/// they read this tick's state and strikes). It is granted og.WeatherState
/// and og.LightningStrike when a game names them ([`enable`]).
pub(crate) fn install(world: &mut World, layouts: &BTreeMap<String, Layout>) -> Result<(), String> {
    let (Some(state), Some(strike), Some(weather)) = (
        layouts.get(STATE).cloned(),
        layouts.get(STRIKE).cloned(),
        layouts.get(WEATHER).cloned(),
    ) else {
        return Ok(());
    };
    // A world without og.Weather (or og.Rng) has no weather to simulate.
    let rng = <og_core::Rng as og_core::Reflect>::NAME;
    let (Ok(weather_id), Ok(_)) = (world.id_by_name(WEATHER), world.id_by_name(rng)) else {
        return Ok(());
    };
    world
        .add_system(
            SystemDesc {
                name: SYSTEM.into(),
                reads: vec![WEATHER.into(), rng.into()],
                ..Default::default()
            },
            move |ctx| run(ctx, &state, &strike, (weather_id, &weather)),
        )
        .map_err(|e| e.reason().to_string())
}

/// The lazy records' grant: og.weather writes the state and sends strikes.
pub(crate) fn enable(world: &mut World, name: &str) -> og_core::Result<()> {
    world.grant(SYSTEM, &[name])
}

fn run(
    ctx: &mut SystemContext<'_>,
    state: &Layout,
    strike: &Layout,
    weather: (SchemaId, &Layout),
) -> og_core::Result<()> {
    let Ok(id) = ctx.id_by_name(STATE) else {
        return Ok(());
    };
    let invalid = |e: String| og_core::Error::new(og_core::ErrorKind::Invalid, e);
    let bytes = ctx.resource_bytes(id)?;
    let mut s = state.decode(&bytes);
    let row = record::WeatherState::from_json(&s).map_err(invalid)?;
    let w: record::Weather = weather
        .1
        .read(&ctx.resource_bytes(weather.0)?)
        .map_err(invalid)?;
    let time = ctx.time();
    let next = step(
        Step {
            wetness: row.wetness,
            puddles: row.puddles,
            snow_depth: row.snow_depth,
            dust: row.dust,
        },
        &row,
        u64::from(w.kind),
        w.intensity,
        time.dt,
    );
    s["wetness"] = json!(next.wetness);
    s["puddles"] = json!(next.puddles);
    s["snowDepth"] = json!(next.snow_depth);
    s["dust"] = json!(next.dust);
    let rate = row.lightning.max(0.0);
    if rate > 0.0 {
        let mut rng = og_core::Rng::default();
        match row.stream {
            // The first draw: forked from the world's generator, so the
            // game's own sequence is not disturbed.
            0 => rng = ctx.resource::<og_core::Rng>()?.fork(SYSTEM),
            kept => rng.restore(kept),
        }
        // A strike this tick with probability rate per minute times dt.
        if (rng.next_f64() as f32) < rate / 60.0 * time.dt {
            let (x, z) = disc(&mut rng);
            let r = row.lightning_radius.max(0.0);
            let at = row.lightning_center + Vec3::new(x * r, 0.0, z * r);
            let n = row.strikes + 1;
            s["strikes"] = json!(n);
            s["lastStrike"] = json!(time.tick as u32);
            s["strikePosition"] = json!({ "x": at.x, "y": at.y, "z": at.z });
            if let Ok(event) = ctx.id_by_name(STRIKE) {
                let e = json!({ "position": { "x": at.x, "y": at.y, "z": at.z }, "strike": n });
                let b = strike.encode(None, Some(&e)).map_err(invalid)?;
                ctx.send_bytes(event, &b)?;
            }
        }
        s["stream"] = json!(rng.save());
    }
    let out = state.encode(Some(&bytes), Some(&s)).map_err(invalid)?;
    if out != bytes {
        ctx.set_resource_bytes(id, &out)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_for(s: Step, kind: u64, intensity: f32, seconds: u32) -> Step {
        let state = record::WeatherState::default();
        (0..seconds * 60).fold(s, |s, _| step(s, &state, kind, intensity, 1.0 / 60.0))
    }

    /// A minute of rain soaks the ground; the puddles wait for it to be
    /// half soaked, then fill in two minutes more; five minutes of dry
    /// weather dries it. Snow lies 1 cm per minute and melts after; rain
    /// melts it three times as fast.
    #[test]
    fn rain_soaks_and_fills_then_dries_and_snow_lies_then_melts() {
        let half = run_for(Step::default(), RAIN, 1.0, 30);
        assert!(
            (half.wetness - 0.5).abs() < 0.01 && half.puddles == 0.0,
            "{half:?}"
        );
        let soaked = run_for(Step::default(), RAIN, 1.0, 60);
        assert!(soaked.wetness > 0.999 && soaked.puddles > 0.2 && soaked.puddles < 0.3);
        let full = run_for(soaked, RAIN, 1.0, 120);
        assert!(full.puddles > 0.999);
        let dry = run_for(full, 0, 1.0, 300);
        assert!(
            dry.wetness < 1e-3 && (dry.puddles - 0.5).abs() < 0.01,
            "{dry:?}"
        );
        let snow = run_for(Step::default(), SNOW, 1.0, 120);
        assert!((snow.snow_depth - 0.02).abs() < 1e-4 && snow.wetness == 0.0);
        let thawed = run_for(snow, 0, 1.0, 120);
        assert!((thawed.snow_depth - 0.012).abs() < 1e-4, "{thawed:?}");
        let rained = run_for(snow, RAIN, 1.0, 120);
        assert!(rained.snow_depth < 1e-4);
    }

    /// A dust storm covers the ground in ten minutes (five at intensity 2)
    /// and the dust stays through dry weather and snow; a minute and a
    /// half of rain washes it off, half of it in 45 s.
    #[test]
    fn a_dust_storm_settles_dust_that_stays_until_rain_washes_it() {
        let half = run_for(Step::default(), DUST, 1.0, 300);
        assert!((half.dust - 0.5).abs() < 0.01, "{half:?}");
        let full = run_for(Step::default(), DUST, 2.0, 300);
        assert!(full.dust > 0.999 && full.wetness == 0.0, "{full:?}");
        let kept = run_for(run_for(full, 0, 1.0, 600), SNOW, 1.0, 60);
        assert!(kept.dust > 0.999, "{kept:?}");
        let washing = run_for(full, RAIN, 1.0, 45);
        assert!((washing.dust - 0.5).abs() < 0.01, "{washing:?}");
        assert!(run_for(full, RAIN, 1.0, 90).dust < 1e-3);
    }

    /// Strikes land evenly in the disc (each quadrant about a quarter, none
    /// outside), and the same stream state gives the same strikes.
    #[test]
    fn strikes_land_in_the_disc_from_their_stream() {
        let mut rng = og_core::Rng::new(7).fork(SYSTEM);
        let mut quadrants = [0u32; 4];
        for _ in 0..4000 {
            let (x, z) = disc(&mut rng);
            assert!(x * x + z * z <= 1.0);
            quadrants[usize::from(x > 0.0) + 2 * usize::from(z > 0.0)] += 1;
        }
        for q in quadrants {
            assert!((850..1150).contains(&q), "{quadrants:?}");
        }
        let kept = rng.save();
        let (mut a, mut b) = (og_core::Rng::default(), og_core::Rng::default());
        a.restore(kept);
        b.restore(kept);
        assert_eq!(disc(&mut a), disc(&mut b));
    }
}
