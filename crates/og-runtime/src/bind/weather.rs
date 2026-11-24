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
