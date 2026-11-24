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
pub(crate) const WIND_COMMON_WGSL: &str = wgsl!("wind_common.wgsl");
/// wind_lean.wgsl: the simulated field's lean (after [`WIND_COMMON_WGSL`]).
pub(crate) const WIND_LEAN_WGSL: &str = wgsl!("wind_lean.wgsl");

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

/// wind_common.wgsl `wind_noise` on the CPU: value noise in [0, 1],
/// repeating every [`GUST_PERIOD`] cells.
fn gust_noise(p: Vec2) -> f32 {
    let lattice = |x: i32, y: i32| {
        let m = GUST_PERIOD as i32 - 1;
        let (a, b) = ((x & m) as u32, (y & m) as u32 ^ 0x9e37_79b9);
        og_core::hash::unit24(og_core::hash::pcg(a ^ og_core::hash::pcg(b)))
    };
    let i = p.floor();
    let f = p - i;
    let u = f * f * (Vec2::splat(3.0) - 2.0 * f);
    let (x, y) = (i.x as i32, i.y as i32);
    let top = lattice(x, y) + (lattice(x + 1, y) - lattice(x, y)) * u.x;
    let bottom = lattice(x, y + 1) + (lattice(x + 1, y + 1) - lattice(x, y + 1)) * u.x;
    top + (bottom - top) * u.y
}

impl Wind {
    /// The wind's lean at world point `at` (x, z) and `tick`, as every
    /// tier without the simulated field takes it (wind_common.wgsl
    /// `wind_lean_analytic`, here on the CPU for what is moved before it is
    /// drawn: og.Sprite.sway's parts): downwind by `strength` times one
    /// plus `gust` times the gusts' share, with the first octave's phase
    /// of the travelling wave there (radians) for what loops in it.
    pub fn lean_at(&self, at: Vec2, tick: u64, tick_rate: f32) -> (Vec2, f32) {
        if self.strength <= 0.0 {
            return (Vec2::ZERO, 0.0);
        }
        let dir = Vec2::new(self.direction.x, self.direction.z).normalize_or(Vec2::X);
        let (phase, offset, _) = wind_at(self, tick, tick_rate);
        let x = std::f32::consts::TAU / WIND_WAVELENGTH * at.dot(dir);
        let wave = ((x - phase[0]).sin() + 0.5 * (2.3 * x + 1.7 - phase[1]).sin()) / 1.5;
        let p = at / GUST_CELL as f32 - Vec2::from(offset);
        let patches = 0.65 * gust_noise(p) + 0.35 * gust_noise(p * 2.0 + Vec2::new(37.0, 11.0));
        let share = (0.5 * wave + 1.4 * (patches - 0.5)).clamp(-1.0, 1.0);
        let lean = dir * self.strength * (1.0 + self.gust.clamp(0.0, 1.0) * share);
        (lean, x - phase[0])
    }
}
