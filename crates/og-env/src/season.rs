//! The time of year (og.Season): the calendar's date as a function of the
//! tick, and what follows from a date, a latitude and a climate: where the
//! sun stands and how long the day is, the air's temperature, and how far
//! the leaves and the grass are through their year. Each is a pure
//! function, so the sky is drawn at the date the simulation reads and any
//! date looks the same on every run.
//!
//! - The sun's declination is Spencer's Fourier series (J. W. Spencer,
//!   "Fourier series representation of the position of the sun", Search
//!   2(5), 1971), as NOAA's solar calculator uses it; its place in the sky
//!   and the day's length are the spherical triangle of the latitude, the
//!   declination and the hour angle, for the sun's centre without
//!   refraction (an equinox day is 12 hours, as og.DayNight's has been).
//! - The temperature is a yearly sine that peaks [`LAG`] days after the
//!   summer solstice, with the day's swing about it.
//! - The leaves follow the seasonal temperature: they open when it rises
//!   through [`BUD`] (a thermal-time threshold for budburst) and turn when
//!   it falls through [`TURN`] (the cold that drives senescence in
//!   Delpierre et al., "Modelling interannual and spatial variability of
//!   leaf senescence for three deciduous tree species in France", Agric.
//!   For. Meteorol. 149, 2009), as closed-form dates on the yearly sine
//!   instead of sums over a weather record, so a date alone gives the
//!   state.
//!
//! The transcendentals are [`og_core::math`]'s: every host computes the
//! same bits.

use og_core::math::{acos, cos, sin_cos};

/// The days in the year.
pub const YEAR: f32 = 365.0;
/// The northern spring equinox, summer solstice, autumn equinox and winter
/// solstice, in days from New Year's midnight.
pub const SPRING: f32 = 79.0;
pub const SUMMER: f32 = 172.0;
pub const AUTUMN: f32 = 265.0;
pub const WINTER: f32 = 355.0;
/// The days the warmest day follows the summer solstice by.
pub const LAG: f32 = 30.0;
/// The seasonal temperature (°C) the leaves open at as it rises.
pub const BUD: f32 = 8.0;
/// The seasonal temperature (°C) the leaves turn at as it falls.
pub const TURN: f32 = 12.0;

const TAU: f32 = std::f32::consts::TAU;
const PI: f32 = std::f32::consts::PI;

/// `day` wrapped into the year, 0..365.
pub fn wrap(day: f32) -> f32 {
    let d = day.rem_euclid(YEAR);
    if d >= YEAR { 0.0 } else { d }
}

/// The calendar's date `seconds` of simulation after tick 0 (days,
/// 0..365): `start` days plus `speed` days per second; a stopped calendar
/// stays at `start`.
pub fn calendar(start: f32, speed: f32, running: bool, seconds: f64) -> f32 {
    let start = f64::from(start);
    let year = f64::from(YEAR);
    if !running {
        return wrap(start.rem_euclid(year) as f32);
    }
    let speed = f64::from(speed);
    wrap((start + seconds * speed).rem_euclid(year) as f32)
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    if b <= a {
        return if x < a { 0.0 } else { 1.0 };
    }
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The sun's declination on `day`, in radians: +0.409 at the northern
/// summer solstice, 0 at the equinoxes (Spencer 1971).
pub fn declination(day: f32) -> f32 {
    let g = TAU * wrap(day) / YEAR;
    let (s1, c1) = sin_cos(g);
    let (s2, c2) = sin_cos(2.0 * g);
    let (s3, c3) = sin_cos(3.0 * g);
    0.006918 - 0.399912 * c1 + 0.070257 * s1 - 0.006758 * c2 + 0.000907 * s2 - 0.002697 * c3
        + 0.00148 * s3
}

/// The latitude the sun's geometry uses, in radians: short of the poles,
/// where every bearing is south.
fn phi(latitude: f32) -> f32 {
    latitude.clamp(-89.0, 89.0).to_radians()
}

/// Towards the sun at `hours` of solar time (12: its highest), for a
/// viewer at `latitude` (degrees north) under a sun of `declination`
/// (radians): its parts towards the east, up and towards the south, a
/// unit vector. Below the horizon its `up` part is negative.
pub fn sun(latitude: f32, declination: f32, hours: f32) -> [f32; 3] {
    let (sp, cp) = sin_cos(phi(latitude));
    let (sd, cd) = sin_cos(declination);
    let (sh, ch) = sin_cos((hours.rem_euclid(24.0) - 12.0) / 24.0 * TAU);
    [-cd * sh, sp * sd + cp * cd * ch, sp * cd * ch - cp * sd]
}

/// When the sun rises and sets, in hours of solar time, and the hours
/// between.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SunTimes {
    pub sunrise: f32,
    pub sunset: f32,
    pub day_length: f32,
}

/// The sun's rising and setting at `latitude` (degrees north) under a sun
/// of `declination` (radians). Where it never sets the day is 24 hours,
/// from 0 to 24; where it never rises the day is none, at noon.
pub fn sun_times(latitude: f32, declination: f32) -> SunTimes {
    let (sp, cp) = sin_cos(phi(latitude));
    let (sd, cd) = sin_cos(declination);
    // The hour angle of the horizon: cos H = -tan φ tan δ.
    let c = -(sp * sd) / (cp * cd);
    let half = if c <= -1.0 {
        12.0
    } else if c >= 1.0 {
        0.0
    } else {
        acos(c) / PI * 12.0
    };
    SunTimes {
        sunrise: 12.0 - half,
        sunset: 12.0 + half,
        day_length: 2.0 * half,
    }
}

/// The date as the place lives it: the same day is the opposite season
/// south of the equator.
pub fn local(day: f32, latitude: f32) -> f32 {
    if latitude < 0.0 {
        wrap(day + YEAR * 0.5)
    } else {
        wrap(day)
    }
}

/// The season at `latitude` on `day`, from 0 up to 4: its whole part is
/// the season (0 spring, 1 summer, 2 autumn, 3 winter), each starting at
/// its equinox or solstice, and its fraction how far through it. South of
/// the equator the same equinoxes and solstices start the opposite
/// seasons.
pub fn season(day: f32, latitude: f32) -> f32 {
    let north = northern_season(day);
    if latitude < 0.0 {
        (north + 2.0) % 4.0
    } else {
        north
    }
}

fn northern_season(day: f32) -> f32 {
    // Days since the spring equinox, against the four seasons' lengths.
    let d = wrap(day - SPRING);
    let starts = [0.0, SUMMER - SPRING, AUTUMN - SPRING, WINTER - SPRING, YEAR];
    for i in 0..4 {
        if d < starts[i + 1] {
            let s = i as f32 + (d - starts[i]) / (starts[i + 1] - starts[i]);
            return s.min(3.999_999);
        }
    }
    0.0
}

/// A place's climate (og.Season's settings), in °C: the year's mean, the
/// warmest month's mean less the coldest's, and the afternoon's
/// temperature less the dawn's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Climate {
    pub mean: f32,
    pub range: f32,
    pub day_range: f32,
}

/// The year's angle from its warmest day, in radians (-π..π).
fn from_peak(day: f32, latitude: f32) -> f32 {
    let d = wrap(local(day, latitude) - (SUMMER + LAG));
    let d = if d > YEAR * 0.5 { d - YEAR } else { d };
    TAU * d / YEAR
}

/// The seasonal temperature on `day`: the mean of its days, without the
/// day's swing.
pub fn seasonal_temperature(c: &Climate, day: f32, latitude: f32) -> f32 {
    c.mean + 0.5 * c.range.max(0.0) * cos(from_peak(day, latitude))
}

/// The air's temperature on `day` at `hours` of solar time: the seasonal
/// temperature with the day's swing, warmest at 15 h.
pub fn temperature(c: &Climate, day: f32, latitude: f32, hours: f32) -> f32 {
    seasonal_temperature(c, day, latitude)
        + 0.5 * c.day_range.max(0.0) * cos((hours.rem_euclid(24.0) - 15.0) / 24.0 * TAU)
}

/// How far a deciduous tree's leaves are through their year, each 0..1.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Leaves {
    /// How far they have opened: 0 from the day they bud, 1 in full leaf
    /// and until the next bud.
    pub out: f32,
    /// How far they have turned to their autumn colour.
    pub turn: f32,
    /// The share that has fallen.
    pub fall: f32,
}

impl Leaves {
    /// The share of its leaves a tree carries: opened and not yet fallen.
    pub fn canopy(&self) -> f32 {
        self.out * (1.0 - self.fall)
    }
}

/// The year of a deciduous tree in a climate: the day its leaves bud, the
/// days from there until they start to turn, and the spans of each stage.
/// What [`leaves`] evaluates, kept apart so a renderer can evaluate it for
/// each tree and leaf at its own day.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LeafYear {
    /// Never cold enough to drop them: in full leaf all year.
    Evergreen,
    /// Never warm enough to open them: bare all year.
    Bare,
    Seasonal {
        /// The day (0..365, as the place lives it) the leaves bud.
        bud: f32,
        /// The days from the bud until the leaves start to turn.
        growing: f32,
        /// The days the leaves take to open.
        opening: f32,
        /// The days they take to turn.
        turning: f32,
        /// The days after the turn starts that the first and the last
        /// leaf fall.
        fall: [f32; 2],
    },
}

/// The leaves' year in climate `c`.
pub fn leaf_year(c: &Climate) -> LeafYear {
    let a = 0.5 * c.range.max(0.0);
    if c.mean - a >= BUD {
        return LeafYear::Evergreen;
    }
    if c.mean + a <= BUD {
        return LeafYear::Bare;
    }
    let days = YEAR / TAU;
    let peak = SUMMER + LAG;
    let bud = peak - acos(((BUD - c.mean) / a).clamp(-1.0, 1.0)) * days;
    // A summer that never reaches TURN turns soon after its warmest day.
    let turn = peak + (acos(((TURN - c.mean) / a).clamp(-1.0, 1.0)) * days).max(15.0);
    let growing = turn - bud;
    let dormant = YEAR - growing;
    LeafYear::Seasonal {
        bud: wrap(bud),
        growing,
        opening: (0.5 * growing).min(21.0),
        turning: (0.3 * dormant).min(18.0),
        fall: [(0.15 * dormant).min(10.0), (0.6 * dormant).min(38.0)],
    }
}

impl LeafYear {
    /// The leaves on `day` as the place lives it (0..365; see [`leaves`]
    /// for a date and a latitude).
    pub fn at(&self, day: f32) -> Leaves {
        match *self {
            LeafYear::Evergreen => Leaves {
                out: 1.0,
                turn: 0.0,
                fall: 0.0,
            },
            LeafYear::Bare => Leaves::default(),
            LeafYear::Seasonal {
                bud,
                growing,
                opening,
                turning,
                fall,
            } => {
                let x = wrap(day - bud);
                if x < growing {
                    Leaves {
                        out: smooth(0.0, opening, x),
                        turn: 0.0,
                        fall: 0.0,
                    }
                } else {
                    let y = x - growing;
                    Leaves {
                        out: 1.0,
                        turn: smooth(0.0, turning, y),
                        fall: smooth(fall[0], fall[1], y),
                    }
                }
            }
        }
    }
}

/// A deciduous tree's leaves on `day` at `latitude` in climate `c`.
pub fn leaves(c: &Climate, day: f32, latitude: f32) -> Leaves {
    leaf_year(c).at(local(day, latitude))
}

/// How far the grass has grown, 0 dormant to 1 full: it grows while the
/// seasonal temperature is above about 5 °C, so it greens before the trees
/// and stays into autumn.
pub fn grass_growth(c: &Climate, day: f32, latitude: f32) -> f32 {
    smooth(2.0, 12.0, seasonal_temperature(c, day, latitude))
}

/// The days a jump back may be and still be reached backward; a longer one
/// runs forward round the year.
const BACK: f32 = 30.0;

/// One tick of the drawn date (og.Season.day) following the calendar: the
/// date after it, given the drawn date `day` (negative before the first
/// tick), the calendar's date now, `advance` the days the calendar's speed
/// moved it this tick, and `change` the seconds a season's days take to
/// play. The drawn date keeps the calendar's pace and closes what a jump
/// left between them at a season every `change` seconds, forward round the
/// year unless the calendar is less than [`BACK`] days behind it. With
/// `change` 0, and on the first tick, it is the calendar's.
pub fn follow(day: f32, calendar: f32, advance: f32, change: f32, dt: f32) -> f32 {
    if day < 0.0 || change <= 0.0 {
        return calendar;
    }
    let mut ahead = wrap(calendar - day);
    if ahead > YEAR - BACK {
        ahead -= YEAR;
    }
    let step = YEAR / 4.0 / change * dt;
    if ahead >= 0.0 {
        let moved = advance.max(0.0) + step;
        if ahead <= moved {
            calendar
        } else {
            wrap(day + moved)
        }
    } else if -ahead <= step {
        calendar
    } else {
        wrap(day - step)
    }
}
