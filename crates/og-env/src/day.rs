//! The time of day (og.DayNight's clock), as a function of the tick: the
//! sky is drawn at the hour the simulation reads.

/// The clock at `tick` (hours, 0..24): `start` hours plus `speed` hours
/// per second of ticks; a stopped clock stays at `start`.
pub fn hours(start: f32, speed: f32, running: bool, tick: u64, dt: f32) -> f32 {
    let start = f64::from(start);
    if !running {
        return start.rem_euclid(24.0) as f32;
    }
    let speed = f64::from(speed);
    (start + tick as f64 * f64::from(dt) * speed).rem_euclid(24.0) as f32
}
