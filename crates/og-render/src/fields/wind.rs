//! og.Wind's field: what every presentation-only wind (og.Grass, the
//! trees) reads to bend in the same gusts (shaders/wind_common.wgsl): its
//! direction, strength and gust, its travelling wave's phases
//! ([`wind_phases`]), how far the gust patches have scrolled
//! downwind, and a clock, each at the tick drawn and the one before it
//! (motion vectors), all reduced in f64 from the tick so frames reproduce
//! and a long session moves as its first minute. Never read by the tick.
//!
//! Ported from bevy_grass v0.20.3 (third_party/bevy-grass). MIT License,
//! Copyright (c) 2026 Gediminas Mikelenas: the full notice is in grass.rs
//! and third_party/bevy-grass/LICENSE.

use glam::{Vec2, Vec3};

/// wind_common.wgsl.
pub(crate) const WIND_COMMON_WGSL: &str = wgsl3d!("wind_common.wgsl");
/// wind_lean.wgsl: the simulated field's lean (after [`WIND_COMMON_WGSL`]).
pub(crate) const WIND_LEAN_WGSL: &str = wgsl3d!("wind_lean.wgsl");

/// Seconds after which the wind's clock starts again (wind_common.wgsl).
const TIME_PERIOD: f64 = 3600.0;
/// The gust noise's cell (m) and period (cells).
pub(crate) const GUST_CELL: f64 = 14.0;
pub(crate) const GUST_PERIOD: f64 = 256.0;

/// `WindField` in wind_common.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct WindFieldGpu {
    pub(crate) dir: [f32; 4],
    pub(crate) phase: [f32; 4],
    pub(crate) offset: [f32; 4],
    pub(crate) time: [f32; 4],
    /// The simulated field's grid (`wind_field.rs` [`crate::wind_field::WindGrid`]).
    pub(crate) grid: [f32; 4],
    pub(crate) grid_outer: [f32; 4],
}

/// og.Wind at `tick`: its wave's phases, how far the gust patches have
/// travelled (cells, within their period) and the clock (within its
/// period), all reduced in f64 so a long session moves as its first minute.
fn wind_at(w: &crate::Wind, tick: u64, rate: f32) -> ([f32; 2], [f32; 2], f32) {
    let t = tick as f64 / f64::from(rate.max(1e-3));
    let dir = Vec2::new(w.direction.x, w.direction.z).normalize_or(Vec2::X);
    let travel = f64::from(w.speed) * t / GUST_CELL;
    let off = |d: f32| (f64::from(d) * travel).rem_euclid(GUST_PERIOD) as f32;
    (
        wind_phases(w, tick, rate),
        [off(dir.x), off(dir.y)],
        t.rem_euclid(TIME_PERIOD) as f32,
    )
}

/// og.Wind's field at `tick`, and as it was at `previous`, read with the
/// simulated field of grid `grid` (`wind_field.rs`).
pub(crate) fn field_gpu(
    wind: Option<crate::Wind>,
    tick: u64,
    previous: u64,
    rate: f32,
    origin: Vec3,
    grid: crate::wind_field::WindGrid,
) -> WindFieldGpu {
    let mut gpu = WindFieldGpu {
        time: [0.0, 0.0, origin.x, origin.z],
        grid: grid[0],
        grid_outer: grid[1],
        ..Default::default()
    };
    if let Some(w) = wind.filter(|w| w.strength > 0.0) {
        let dir = Vec2::new(w.direction.x, w.direction.z).normalize_or(Vec2::X);
        let (p, o, t) = wind_at(&w, tick, rate);
        let (pp, po, pt) = wind_at(&w, previous, rate);
        gpu.dir = [dir.x, dir.y, w.strength, w.gust.clamp(0.0, 1.0)];
        gpu.phase = [p[0], p[1], pp[0], pp[1]];
        gpu.offset = [o[0], o[1], po[0], po[1]];
        gpu.time = [t, pt, origin.x, origin.z];
    }
    gpu
}

/// The wind the renderer sways vegetation with ([`crate::RenderScene::wind`]):
/// [`ScatterDraw`] copies lean downwind and sway in gusts that sweep
/// across the field (the module's documentation has the shape). Visual
/// only, a function of the tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wind {
    /// The mean lean: how far a copy's tip moves downwind per metre of its
    /// height (0.2: a 1 m stem's tip leans 20 cm); 0 is still.
    pub strength: f32,
    /// Where it blows: its horizontal part, normalised.
    pub direction: Vec3,
    /// How fast the gusts travel downwind (m/s).
    pub speed: f32,
    /// How much the lean varies in the gusts, 0..1: 0 a steady lean, 1
    /// from upright to twice the mean.
    pub gust: f32,
    /// How much the simulated field (`wind_field.rs`) swirls on its own,
    /// 0..1: curl noise of the gusts' size, at this share of the speed.
    pub turbulence: f32,
    /// The gust patches' size (m).
    pub gust_size: f32,
}

impl Wind {
    /// How far a stem's tip leans downwind per metre of its height, for
    /// each m/s the air moves: what og.Wind's speed and an og.WindMotor's
    /// push lean vegetation by.
    pub const LEAN_PER_MPS: f32 = 0.05;

    /// The wind of air moving at `speed` m/s along `direction`, with the
    /// default gusts: vegetation leans by [`Wind::LEAN_PER_MPS`] of it.
    pub fn blowing(direction: Vec3, speed: f32) -> Wind {
        Wind {
            strength: Wind::LEAN_PER_MPS * speed,
            direction,
            speed,
            ..Wind::default()
        }
    }
}

impl Default for Wind {
    fn default() -> Self {
        Wind {
            strength: 0.2,
            direction: Vec3::X,
            speed: 3.0,
            gust: 0.6,
            turbulence: 0.3,
            gust_size: 14.0,
        }
    }
}

/// The length of the wind's gust wave (metres).
pub const WIND_WAVELENGTH: f32 = 10.0;

/// The wind's second octave: its wavenumber and time phase over the
/// first's (scatter_common.wgsl `scatter_sway`).
const WIND_OCTAVE: f64 = 2.3;

/// The wind's two time phases at `tick` (radians, reduced in f64): the
/// first octave's `k speed t` and the second's.
pub fn wind_phases(wind: &Wind, tick: u64, tick_rate: f32) -> [f32; 2] {
    let t = tick as f64 / f64::from(tick_rate.max(1e-3));
    let k = std::f64::consts::TAU / f64::from(WIND_WAVELENGTH);
    let first = k * f64::from(wind.speed) * t;
    [first, first * WIND_OCTAVE].map(|p| p.rem_euclid(std::f64::consts::TAU) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The wind's clock and gusts are reduced in f64: a tick a day into a
    /// session lands where its clock says, and the same tick is the same.
    #[test]
    fn the_wind_is_a_function_of_the_tick() {
        let w = crate::Wind::default();
        // A day and ten seconds in.
        let tick = 5_184_600;
        let a = field_gpu(
            Some(w),
            tick,
            tick - 1,
            60.0,
            Vec3::ZERO,
            crate::wind_field::NO_GRID,
        );
        let b = field_gpu(
            Some(w),
            tick,
            tick - 1,
            60.0,
            Vec3::ZERO,
            crate::wind_field::NO_GRID,
        );
        assert_eq!(a, b);
        // A day in, the clock is within its period and one tick moved it by 1/60 s.
        assert!((a.time[0] - 10.0).abs() < 1e-4, "{}", a.time[0]);
        assert!((a.time[0] - a.time[1] - 1.0 / 60.0).abs() < 1e-4);
        // 3 m/s along +x for 86410 s, in cells, within the period.
        let cells = (3.0 * 86_410.0 / GUST_CELL).rem_euclid(GUST_PERIOD) as f32;
        assert!(
            (a.offset[0] - cells).abs() < 1e-3,
            "{} {cells}",
            a.offset[0]
        );
        assert_eq!(a.offset[1], 0.0);
        let still = field_gpu(None, 90, 89, 60.0, Vec3::ZERO, crate::wind_field::NO_GRID);
        assert_eq!(still.dir[2], 0.0);
    }
}
