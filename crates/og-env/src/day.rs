//! The time of day (og.DayNight's clock), as a function of the simulated
//! time: the sky is drawn at the hour the simulation reads.

/// The clock `seconds` of simulation after tick 0 (hours, 0..24): `start`
/// hours plus `speed` hours per second; a stopped clock stays at `start`.
/// In f64 throughout, as the game's own clocks are, so an hour a game wrote
/// reads back as written.
pub fn hours(start: f64, speed: f64, running: bool, seconds: f64) -> f64 {
    if !running {
        return start.rem_euclid(24.0);
    }
    (start + seconds * speed).rem_euclid(24.0)
}
