//! og.season: og.Season's date as the world lives it. The calendar is a
//! function of the tick (`og_env::season::calendar`); the tick keeps only
//! the date drawn and read (og.Season.day), which follows the calendar
//! through a jump over og.Season.change. Everything else, the sun's place,
//! the temperature, the leaves' year, is a pure function of that date
//! ([`Now`]), which extraction draws and og.atmosphere.season answers, so
//! what a game reads is what the frame shows. og.Season is lazy
//! (crate::engine::lazy): until a game names it this system finds nothing
//! to do and the world keeps its type table and hash. `og_core::math`'s
//! transcendentals only, so every host computes the same bits.

use crate::engine::engine;
use crate::host::codec::{Layout, RecordKind};
use crate::host::wire::{FromJson, SeasonInfo, record};
use og_ecs::{SystemContext, SystemDesc, World};
use og_env::season::{self, Climate};
use serde_json::json;
use std::collections::BTreeMap;

pub(crate) const STATE: &str = "og.Season";
pub(crate) const SYSTEM: &str = "og.season";
const DAY_NIGHT: &str = "og.DayNight";

/// Schedules og.season (early in the tick, before the game's systems, so
/// they read this tick's date). It is granted og.Season when a game names
/// it ([`enable`]).
pub(crate) fn install(world: &mut World, layouts: &BTreeMap<String, Layout>) -> Result<(), String> {
    let Some(state) = layouts.get(STATE).cloned() else {
        return Ok(());
    };
    world
        .add_system(
            SystemDesc {
                name: SYSTEM.into(),
                ..Default::default()
            },
            move |ctx| run(ctx, &state),
        )
        .map_err(|e| e.reason().to_string())
}

/// The lazy record's grant: og.season writes the drawn date.
pub(crate) fn enable(world: &mut World) -> og_core::Result<()> {
    world.grant(SYSTEM, &[STATE])
}

fn run(ctx: &mut SystemContext<'_>, state: &Layout) -> og_core::Result<()> {
    let Ok(id) = ctx.id_by_name(STATE) else {
        return Ok(());
    };
    let invalid = |e: String| og_core::Error::new(og_core::ErrorKind::Invalid, e);
    let bytes = ctx.resource_bytes(id)?;
    let mut s = state.decode(&bytes);
    let row = record::Season::from_json(&s).map_err(invalid)?;
    let time = ctx.time();
    // A year that is off keeps no date: enabled again, it starts on the
    // calendar's.
    let day = if row.enabled {
        let calendar = season::calendar(row.start, row.speed, true, time.tick, time.dt);
        season::follow(
            row.day,
            calendar,
            row.speed.max(0.0) * time.dt,
            row.change,
            time.dt,
        )
    } else {
        -1.0
    };
    if day.to_bits() == row.day.to_bits() {
        return Ok(());
    }
    s["day"] = json!(day);
    let out = state.encode(Some(&bytes), Some(&s)).map_err(invalid)?;
    ctx.set_resource_bytes(id, &out)
}

/// og.Season at a tick: the date drawn and read, and where and in what
/// climate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Now {
    pub day: f32,
    pub latitude: f32,
    pub climate: Climate,
    /// Whether it sets the sun's path (enabled, with `sun`).
    pub sun: bool,
}

impl Now {
    /// `s` at `tick`: the date og.season last wrote while a change plays
    /// (Season.change above 0), else the calendar's, which it then is: the
    /// calendar's before the first tick, `start` while the year is off.
    pub(crate) fn of(s: &record::Season, tick: u64, dt: f32) -> Now {
        let day = if s.enabled && s.change > 0.0 && s.day >= 0.0 {
            season::wrap(s.day)
        } else {
            season::calendar(s.start, s.speed, s.enabled, tick, dt)
        };
        Now {
            day,
            latitude: s.latitude.clamp(-90.0, 90.0),
            climate: Climate {
                mean: s.temperature,
                range: s.temperature_range,
                day_range: s.day_range,
            },
            sun: s.enabled && s.sun,
        }
    }

    /// Towards the sun at `hours` of solar time, its parts towards the
    /// east, up and towards the south (`og_env::season::sun`).
    pub(crate) fn sun(&self, hours: f32) -> [f32; 3] {
        season::sun(self.latitude, season::declination(self.day), hours)
    }

    /// og.atmosphere.season's answer, at og.DayNight's `hours` when its
    /// cycle runs.
    pub(crate) fn info(&self, hours: Option<f32>) -> SeasonInfo {
        let declination = season::declination(self.day);
        let times = season::sun_times(self.latitude, declination);
        let leaves = season::leaves(&self.climate, self.day, self.latitude);
        SeasonInfo {
            day: self.day,
            season: season::season(self.day, self.latitude),
            declination: declination.to_degrees(),
            sunrise: times.sunrise,
            sunset: times.sunset,
            day_length: times.day_length,
            temperature: match hours {
                Some(h) => season::temperature(&self.climate, self.day, self.latitude, h),
                None => season::seasonal_temperature(&self.climate, self.day, self.latitude),
            },
            leaf_out: leaves.out,
            leaf_turn: leaves.turn,
            leaf_fall: leaves.fall,
            grass_growth: season::grass_growth(&self.climate, self.day, self.latitude),
        }
    }
}

/// og.DayNight's clock when its cycle runs.
fn day_hours(d: &record::DayNight, tick: u64, dt: f32) -> Option<f32> {
    d.enabled
        .then(|| og_env::day::hours(d.start, d.speed, true, tick, dt))
}

/// og.Season and og.DayNight's hour in `world`, read between ticks: `None`
/// for a world that never named og.Season.
pub(crate) fn of_world(world: &World, layouts: &BTreeMap<String, Layout>) -> Option<SeasonInfo> {
    let time = world.time();
    let s = crate::present::fx::resource(world, layouts.get(STATE))?;
    let s = record::Season::from_json(&s).ok()?;
    let hours = crate::present::fx::resource(world, layouts.get(DAY_NIGHT))
        .and_then(|d| record::DayNight::from_json(&d).ok())
        .and_then(|d| day_hours(&d, time.tick, time.dt));
    Some(Now::of(&s, time.tick, time.dt).info(hours))
}

// ── og.atmosphere.season ────────────────────────────────────────────────

pub fn og_atmosphere_season(world: u32) -> SeasonInfo {
    let defaults = || Now::of(&record::Season::default(), 0, 0.0).info(None);
    let Ok(eng) = engine(world) else {
        return defaults();
    };
    // Inside a system, through its context (og.Season declared among its
    // reads, and og.DayNight for the hour); between ticks, from the world.
    if let Some(frame) = crate::engine::running(world) {
        // SAFETY: the frame is published while this system runs.
        let ctx = unsafe { frame.ctx() };
        let time = ctx.time();
        let Some(s) = crate::host::api::running_record(&eng, STATE, RecordKind::Resource)
            .ok()
            .and_then(|(l, id)| l.read::<record::Season>(&ctx.resource_bytes(id).ok()?).ok())
        else {
            return defaults();
        };
        let hours = crate::host::api::running_record(&eng, DAY_NIGHT, RecordKind::Resource)
            .ok()
            .and_then(|(l, id)| {
                l.read::<record::DayNight>(&ctx.resource_bytes(id).ok()?)
                    .ok()
            })
            .and_then(|d| day_hours(&d, time.tick, time.dt));
        return Now::of(&s, time.tick, time.dt).info(hours);
    }
    let Ok(w) = eng.world() else {
        return defaults();
    };
    let layouts = &eng.state.borrow().layouts;
    of_world(w, layouts).unwrap_or_else(defaults)
}
