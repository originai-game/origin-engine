//! The weather on the ground: what falls (og.Weather's kind) and what it
//! leaves behind, wetness, puddles, lying snow and dust, as one rule of
//! the tick. The renderer draws the kinds it names; the simulation steps
//! the ground with [`step`] and lands lightning with [`disc`].
//!
//! Plain f32 arithmetic only (no transcendental functions), so every host
//! computes the same bits.

/// Clear weather.
pub const NONE: u32 = 0;
/// Rain streaks falling with the wind, dimmed light, a grey distance fog.
pub const RAIN: u32 = 1;
/// Flakes drifting down on gusts, a pale fog.
pub const SNOW: u32 = 2;
/// Dust blowing along the wind near the ground, a warm thick fog.
pub const DUST: u32 = 3;
/// No particles, a dense low height fog.
pub const MIST: u32 = 4;

/// What the weather has left on the ground (og.WeatherState's numbers).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ground {
    pub wetness: f32,
    pub puddles: f32,
    pub snow_depth: f32,
    pub dust: f32,
}

/// How fast the ground follows the weather (og.WeatherState's settings):
/// times in seconds, snowfall and melt in metres per minute.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rates {
    pub soak_time: f32,
    pub puddle_time: f32,
    pub dry_time: f32,
    pub drain_time: f32,
    pub snowfall: f32,
    pub melt: f32,
    pub dust_time: f32,
    pub wash_time: f32,
}

/// The rules: rain soaks the ground over soakTime and, once it is more than
/// half soaked, fills the puddles over puddleTime; without rain both dry
/// (dryTime, drainTime). Snow settles at snowfall per minute and melts at
/// melt per minute when none falls, three times as fast in rain; snow
/// keeps the ground under it from drying no faster than it would. A dust
/// storm settles dust over dustTime; it stays until rain washes it off
/// over washTime.
pub fn step(s: Ground, rates: &Rates, kind: u32, intensity: f32, dt: f32) -> Ground {
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
        o.wetness += dt * rain / rates.soak_time.max(0.1);
        if s.wetness > 0.5 {
            o.puddles += dt * rain / rates.puddle_time.max(0.1);
        }
    } else {
        o.wetness -= dt / rates.dry_time.max(0.1);
        o.puddles -= dt / rates.drain_time.max(0.1);
    }
    if snowing > 0.0 {
        o.snow_depth += dt * snowing * rates.snowfall.max(0.0) / 60.0;
    } else {
        o.snow_depth -= dt * rates.melt.max(0.0) / 60.0 * (1.0 + 2.0 * rain);
    }
    if kind == DUST {
        o.dust += dt * intensity.clamp(0.0, 2.0) / rates.dust_time.max(0.1);
    }
    o.dust -= dt * rain / rates.wash_time.max(0.1);
    o.dust = o.dust.clamp(0.0, 1.0);
    o.wetness = o.wetness.clamp(0.0, 1.0);
    o.puddles = o.puddles.clamp(0.0, 1.0);
    o.snow_depth = o.snow_depth.max(0.0);
    o
}

/// A strike's offset in the unit disc: rejection sampling, no trig.
pub fn disc(rng: &mut og_core::Rng) -> (f32, f32) {
    loop {
        let x = rng.range_f32(-1.0, 1.0);
        let z = rng.range_f32(-1.0, 1.0);
        if x * x + z * z <= 1.0 {
            return (x, z);
        }
    }
}
