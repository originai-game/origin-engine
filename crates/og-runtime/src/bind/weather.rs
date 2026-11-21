//! og.weather (the R4 round): og.Weather drawn and og.WeatherState
//! simulated in the tick. The weather drawn (og.Weather's readonly fields,
//! `og_env::weather::Falling`) follows the game's kind and intensity over
//! og.Weather.change, so a switch builds the new weather as the old one
//! thins out; the ground (wetness, puddles, lying snow and dust) follows
//! the weather drawn, and lightning strikes at random ticks from a stream
//! forked from og.Rng and kept in the state, so a replay, a restore and
//! every host strike alike. og.WeatherState and og.LightningStrike are lazy
//! (crate::engine::lazy): until a game names one, this system draws the
//! weather alone and the world keeps its type table. Plain f32 arithmetic
//! only (no transcendental functions), so every host computes the same
//! bits.

use crate::host::codec::Layout;
use crate::host::wire::{FromJson, record};
use glam::Vec3;
use og_ecs::{SchemaId, SystemContext, SystemDesc, World};
use og_env::weather::{Falling, Ground, Rates, disc, step};
use serde_json::json;
use std::collections::BTreeMap;

pub(crate) const STATE: &str = "og.WeatherState";
pub(crate) const STRIKE: &str = "og.LightningStrike";
pub(crate) const SYSTEM: &str = "og.weather";
const WEATHER: &str = "og.Weather";
/// og.WeatherState and its event, registered together (crate::engine::lazy).
pub(crate) const RECORDS: [&str; 2] = [STATE, STRIKE];

/// og.WeatherState's settings as the ground's rates.
fn rates(state: &record::WeatherState) -> Rates {
    Rates {
        soak_time: state.soak_time,
        puddle_time: state.puddle_time,
        dry_time: state.dry_time,
        drain_time: state.drain_time,
        puddle_limit: state.puddle_limit,
        snowfall: state.snowfall,
        melt: state.melt,
        dust_time: state.dust_time,
        wash_time: state.wash_time,
    }
}

/// Schedules og.weather, after the game's systems (it is added after
/// them): the weather they set this tick is drawn from it, the first tick
/// taking it as it is, and they read the state and strikes it left. It
/// writes og.Weather's drawn fields, and is granted og.WeatherState and
/// og.LightningStrike when a game names them ([`enable`]).
pub(crate) fn install(world: &mut World, layouts: &BTreeMap<String, Layout>) -> Result<(), String> {
    let (Some(state), Some(strike), Some(weather)) = (
        layouts.get(STATE).cloned(),
        layouts.get(STRIKE).cloned(),
        layouts.get(WEATHER).cloned(),
    ) else {
        return Ok(());
    };
    // A world without og.Weather has no weather to draw.
    let Ok(weather_id) = world.id_by_name(WEATHER) else {
        return Ok(());
    };
    let rng = <og_core::Rng as og_core::Reflect>::NAME;
    let reads = world
        .id_by_name(rng)
        .map_or(vec![], |_| vec![rng.to_string()]);
    world
        .add_system(
            SystemDesc {
                name: SYSTEM.into(),
                reads,
                writes: vec![WEATHER.into()],
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

/// The weather og.Weather draws: its readonly fields, or before the first
/// tick wrote them, the game's weather as it is.
pub(crate) fn drawn(w: &record::Weather) -> Falling {
    if w.drawn_intensity < 0.0 {
        return Falling::now(w.kind, w.intensity);
    }
    Falling {
        kind: w.drawn_kind,
        intensity: w.drawn_intensity,
        fading_kind: w.fading_kind,
        fading_intensity: w.fading_intensity,
        settling: w.settling,
    }
}

/// One tick of og.Weather's drawn weather towards the game's: written back
/// only when it moved.
fn draw(
    ctx: &mut SystemContext<'_>,
    (id, layout): (SchemaId, &Layout),
    dt: f32,
) -> Result<Falling, String> {
    let bytes = ctx.resource_bytes(id).map_err(|e| e.to_string())?;
    let w: record::Weather = layout.read(&bytes)?;
    let next = if w.drawn_intensity < 0.0 {
        drawn(&w)
    } else {
        drawn(&w).follow(w.kind, w.intensity, w.change, dt)
    };
    if w.drawn_intensity >= 0.0 && next == drawn(&w) {
        return Ok(next);
    }
    let mut s = layout.decode(&bytes);
    s["drawnKind"] = json!(next.kind);
    s["drawnIntensity"] = json!(next.intensity);
    s["fadingKind"] = json!(next.fading_kind);
    s["fadingIntensity"] = json!(next.fading_intensity);
    s["settling"] = json!(next.settling);
    let out = layout.encode(Some(&bytes), Some(&s))?;
    ctx.set_resource_bytes(id, &out)
        .map_err(|e| e.to_string())?;
    Ok(next)
}

fn run(
    ctx: &mut SystemContext<'_>,
    state: &Layout,
    strike: &Layout,
    weather: (SchemaId, &Layout),
) -> og_core::Result<()> {
    let invalid = |e: String| og_core::Error::new(og_core::ErrorKind::Invalid, e);
    let time = ctx.time();
    let falling = draw(ctx, weather, time.dt()).map_err(invalid)?;
    let Ok(id) = ctx.id_by_name(STATE) else {
        return Ok(());
    };
    let bytes = ctx.resource_bytes(id)?;
    let mut s = state.decode(&bytes);
    let row = record::WeatherState::from_json(&s).map_err(invalid)?;
    let next = step(
        Ground {
            wetness: row.wetness,
            puddles: row.puddles,
            snow_depth: row.snow_depth,
            dust: row.dust,
        },
        &rates(&row),
        &falling,
        time.dt(),
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
        if (rng.next_f64() as f32) < rate / 60.0 * time.dt() {
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
    use og_env::weather::{DUST, RAIN, SNOW};

    /// The ground under og.WeatherState's default settings.
    fn run_for(s: Ground, kind: u32, intensity: f32, seconds: u32) -> Ground {
        let rates = rates(&record::WeatherState::default());
        let falling = Falling::now(kind, intensity);
        (0..seconds * 60).fold(s, |s, _| step(s, &rates, &falling, 1.0 / 60.0))
    }

    /// A minute of rain soaks the ground; the puddles wait for it to be
    /// half soaked, then fill in two minutes more; five minutes of dry
    /// weather dries it. Snow lies 1 cm per minute and melts after; rain
    /// melts it three times as fast.
    #[test]
    fn rain_soaks_and_fills_then_dries_and_snow_lies_then_melts() {
        let half = run_for(Ground::default(), RAIN, 1.0, 30);
        assert!(
            (half.wetness - 0.5).abs() < 0.01 && half.puddles == 0.0,
            "{half:?}"
        );
        let soaked = run_for(Ground::default(), RAIN, 1.0, 60);
        assert!(soaked.wetness > 0.999 && soaked.puddles > 0.2 && soaked.puddles < 0.3);
        let full = run_for(soaked, RAIN, 1.0, 120);
        assert!(full.puddles > 0.999);
        let dry = run_for(full, 0, 1.0, 300);
        assert!(
            dry.wetness < 1e-3 && (dry.puddles - 0.5).abs() < 0.01,
            "{dry:?}"
        );
        let snow = run_for(Ground::default(), SNOW, 1.0, 120);
        assert!((snow.snow_depth - 0.02).abs() < 1e-4 && snow.wetness == 0.0);
        let thawed = run_for(snow, 0, 1.0, 120);
        assert!((thawed.snow_depth - 0.012).abs() < 1e-4, "{thawed:?}");
        let rained = run_for(snow, RAIN, 1.0, 120);
        assert!(rained.snow_depth < 1e-4);
    }

    /// A street that drains: with puddleLimit 0.45, three minutes of steady
    /// rain fill the puddles to 0.45 and no further; the limit lowered to
    /// 0.2 in the rain drains them down to it at drainTime's rate (0.15 in
    /// 90 s of the default 600 s), never below it.
    #[test]
    fn steady_rain_fills_the_puddles_to_their_limit() {
        let mut rates = rates(&record::WeatherState::default());
        rates.puddle_limit = 0.45;
        let rain = Falling::now(RAIN, 1.0);
        let run = |s: Ground, rates: &Rates, seconds: u32| {
            (0..seconds * 60).fold(s, |s, _| step(s, rates, &rain, 1.0 / 60.0))
        };
        let full = run(Ground::default(), &rates, 180);
        assert!((full.puddles - 0.45).abs() < 1e-6, "{full:?}");
        rates.puddle_limit = 0.2;
        let draining = run(full, &rates, 90);
        assert!((draining.puddles - 0.3).abs() < 1e-3, "{draining:?}");
        let drained = run(full, &rates, 300);
        assert!((drained.puddles - 0.2).abs() < 1e-6, "{drained:?}");
    }

    /// The weather drawn over `seconds` from `from`, the game's weather
    /// `kind` at `intensity`, at og.Weather's default change (20 s).
    fn follow_for(from: Falling, kind: u32, intensity: f32, seconds: f32) -> Falling {
        let ticks = (seconds * 60.0).round() as u32;
        (0..ticks).fold(from, |f, _| f.follow(kind, intensity, 20.0, 1.0 / 60.0))
    }

    /// A switch from rain to snow at 20 s a unit: half way the rain has
    /// thinned to 0.5 as the snow built to 0.5 (never a cut, nor both at
    /// once); just past 20 s the snow is full and the rain gone, its last drops
    /// still falling, and 8 s on it is dropped. Turning back half way takes
    /// the rain back where it was; a third kind drops the fainter; change 0
    /// switches at once. The ground soaks only as the rain still falls: half
    /// way through a switch from rain, it soaks at half the rate.
    #[test]
    fn a_switch_thins_one_weather_out_as_the_next_builds() {
        let rain = Falling::now(RAIN, 1.0);
        let half = follow_for(rain, SNOW, 1.0, 10.0);
        assert!(
            half.kind == SNOW
                && half.fading_kind == RAIN
                && (half.intensity - 0.5).abs() < 1e-3
                && (half.fading_intensity - 0.5).abs() < 1e-3,
            "{half:?}"
        );
        let snow = follow_for(rain, SNOW, 1.0, 20.1);
        assert!(
            (snow.intensity - 1.0).abs() < 1e-3
                && snow.fading_kind == RAIN
                && snow.fading_intensity == 0.0,
            "{snow:?}"
        );
        let settled = follow_for(snow, SNOW, 1.0, 8.5);
        assert_eq!(settled, Falling::now(SNOW, 1.0));
        let back = follow_for(half, RAIN, 1.0, 5.0);
        assert!(
            back.kind == RAIN
                && (back.intensity - 0.75).abs() < 1e-3
                && back.fading_kind == SNOW
                && (back.fading_intensity - 0.25).abs() < 1e-3,
            "{back:?}"
        );
        let early = follow_for(rain, SNOW, 1.0, 4.0);
        let third = follow_for(early, DUST, 1.0, 1.0);
        assert!(
            third.kind == DUST && third.fading_kind == RAIN,
            "the fainter snow is dropped: {third:?}"
        );
        assert_eq!(
            rain.follow(SNOW, 2.0, 0.0, 1.0 / 60.0),
            Falling::now(SNOW, 2.0)
        );
        let rates = rates(&record::WeatherState::default());
        let soaked = |f: &Falling| step(Ground::default(), &rates, f, 1.0).wetness;
        assert!((soaked(&half) - soaked(&rain) * 0.5).abs() < 1e-6);
    }

    /// A dust storm covers the ground in ten minutes (five at intensity 2)
    /// and the dust stays through dry weather and snow; a minute and a
    /// half of rain washes it off, half of it in 45 s.
    #[test]
    fn a_dust_storm_settles_dust_that_stays_until_rain_washes_it() {
        let half = run_for(Ground::default(), DUST, 1.0, 300);
        assert!((half.dust - 0.5).abs() < 0.01, "{half:?}");
        let full = run_for(Ground::default(), DUST, 2.0, 300);
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
