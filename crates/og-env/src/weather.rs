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
    pub puddle_limit: f32,
    pub snowfall: f32,
    pub melt: f32,
    pub dust_time: f32,
    pub wash_time: f32,
}

/// How long a faded weather's last particles keep falling after it thins to
/// nothing, in seconds: the longest-lived, the snow's 7 s, reach the ground.
pub const SETTLE: f32 = 8.0;

/// The weather drawn and simulated (og.Weather's readonly fields): the one
/// following the game's kind and intensity, and the one it replaced,
/// thinning out at the same rate, then its last particles falling for
/// [`SETTLE`] seconds.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Falling {
    pub kind: u32,
    pub intensity: f32,
    pub fading_kind: u32,
    pub fading_intensity: f32,
    pub settling: f32,
}

impl Falling {
    /// `kind` at `intensity` at once, nothing fading.
    pub fn now(kind: u32, intensity: f32) -> Falling {
        Falling {
            kind,
            intensity: if kind == NONE {
                0.0
            } else {
                intensity.clamp(0.0, 2.0)
            },
            ..Falling::default()
        }
    }

    /// One tick of `dt` towards the game's `kind` at `intensity`, the
    /// intensity moving by 1 every `change` seconds (0: at once). A new
    /// kind starts from nothing and the one drawn so far thins out; a
    /// return to the kind thinning out takes it back where it is. A third
    /// kind while two are drawn drops the fainter of them.
    pub fn follow(self, kind: u32, intensity: f32, change: f32, dt: f32) -> Falling {
        if change <= 0.0 {
            return Falling::now(kind, intensity);
        }
        let target = Falling::now(kind, intensity).intensity;
        let mut f = self;
        if kind != f.kind {
            if kind == f.fading_kind {
                (f.kind, f.fading_kind) = (f.fading_kind, f.kind);
                (f.intensity, f.fading_intensity) = (f.fading_intensity, f.intensity);
                f.settling = SETTLE;
            } else {
                if f.kind != NONE && (f.fading_kind == NONE || f.intensity >= f.fading_intensity) {
                    f.fading_kind = f.kind;
                    f.fading_intensity = f.intensity;
                    f.settling = SETTLE;
                }
                f.kind = kind;
                f.intensity = 0.0;
            }
        }
        let by = dt / change;
        f.intensity = if f.intensity < target {
            (f.intensity + by).min(target)
        } else {
            (f.intensity - by).max(target)
        };
        if f.fading_kind == NONE {
            f.settling = 0.0;
        } else if f.fading_intensity > 0.0 {
            f.fading_intensity = (f.fading_intensity - by).max(0.0);
        } else {
            f.settling -= dt;
            if f.settling <= 0.0 {
                f.fading_kind = NONE;
                f.settling = 0.0;
            }
        }
        f
    }

    /// How much of `kind` falls, of both weathers drawn.
    pub fn of(&self, kind: u32) -> f32 {
        let at = |k: u32, i: f32| if k == kind { i } else { 0.0 };
        at(self.kind, self.intensity) + at(self.fading_kind, self.fading_intensity)
    }
}

/// The rules: rain soaks the ground over soakTime and, once it is more than
/// half soaked, fills the puddles over puddleTime up to puddleLimit (above
/// it they drain down to it over drainTime); without rain both dry
/// (dryTime, drainTime). Snow settles at snowfall per minute and melts at
/// melt per minute when none falls, three times as fast in rain; snow
/// keeps the ground under it from drying no faster than it would. A dust
/// storm settles dust over dustTime; it stays until rain washes it off
/// over washTime. What falls is both weathers drawn ([`Falling::of`]).
pub fn step(s: Ground, rates: &Rates, falling: &Falling, dt: f32) -> Ground {
    let rain = falling.of(RAIN).clamp(0.0, 2.0);
    let snowing = falling.of(SNOW).clamp(0.0, 2.0);
    let dusting = falling.of(DUST).clamp(0.0, 2.0);
    let mut o = s;
    if rain > 0.0 {
        o.wetness += dt * rain / rates.soak_time.max(0.1);
        let limit = rates.puddle_limit.clamp(0.0, 1.0);
        if s.puddles > limit {
            o.puddles = (s.puddles - dt / rates.drain_time.max(0.1)).max(limit);
        } else if s.wetness > 0.5 {
            o.puddles = (s.puddles + dt * rain / rates.puddle_time.max(0.1)).min(limit);
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
    o.dust += dt * dusting / rates.dust_time.max(0.1);
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
