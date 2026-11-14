//! The aurora (og.Aurora): curtains of emission in the upper atmosphere,
//! drawn by the sky pass over the procedural sky and under the clouds.
//!
//! A port of Lawlor & Genetti, "Interactive Volume Rendering Aurora on the
//! GPU" (Journal of WSCG 19, 2011, pp. 25–32): the display is emissive and
//! factors into a 2D electron-flux footprint on the planet and a
//! height-dependent energy deposition function. Here, as there:
//! - the curtains' paths are curves along the auroral oval (the paper's
//!   splines), folded by travelling waves;
//! - their small-scale complexity comes from a 2D Stam stable-fluids
//!   simulation of the Kelvin–Helmholtz instability of a thin sheared
//!   sheet, run in a long periodic strip and wrapped along each curve
//!   (the paper's section 2), whose early steps give quiet arcs and later
//!   ones the curls of an active display ([`Aurora::activity`]);
//! - the footprint carries a distance field that lets the sky's march
//!   take long steps between curtains (section 3.2); the curves are known
//!   exactly, so the distance is computed from them instead of by jump
//!   flooding an image;
//! - the deposition is the Lazarev model (Lummerzheim 1992, as the paper
//!   uses it) over the U.S. Standard Atmosphere 1976 (public domain),
//!   pre-evaluated into a table by altitude and the precipitation's
//!   characteristic energy (section 2.1), summed over a Maxwellian
//!   spectrum.
//!
//! The emission lines and their heights follow Whiter, Partamies,
//! Gustavsson & Kauristie, "The altitude of green OI 557.7 nm and blue
//! N₂⁺ 427.8 nm aurora", Ann. Geophys. 41, 1–12, 2023 (CC BY 4.0): both
//! peak at about 114 km; the prompt nitrogen bands (427.8 nm and the red
//! first positive band) follow the ionisation and light the lower border;
//! the green line is quenched below about 95 km; the red 630.0 nm line,
//! long-lived and quenched by collisions lower down, glows above 200 km
//! and is spread across the curtain by the time its atoms take to emit.
//! The lines' colours are their CIE 1931 2° colour matching values in
//! linear sRGB, desaturated into the gamut.
//!
//! The footprint is drawn each frame on the GPU (`aurora_footprint.wgsl`)
//! about the viewer, in a square-root radial warp so the texels near the
//! viewer are finest; everything is a function of the tick's clock and
//! the seed, so the display is the same on every run and after a restore.
//! Nothing in the simulation reads it. Its texture is made the first frame
//! with an aurora: skies without one keep a 1×1 placeholder.

use glam::{Quat, Vec2, Vec3};

mod emission;
mod map;
mod strip;

#[cfg(test)]
use emission::emission_rows;
use emission::{line_colors, table};
pub(crate) use map::{AuroraMap, FOOTPRINT_WGSL};
use strip::{kh_strip, strip_mips};

/// The vec4s of the frame uniform's aurora block (`frame.aurora`).
pub(crate) const AURORA_VEC4S: usize = 5;
/// The footprint's texels per side.
pub(crate) const FOOTPRINT: u32 = 1024;
/// The deposition table's rows (characteristic energies), below the
/// footprint in the same texture.
pub(crate) const TABLE_ROWS: u32 = 8;
/// The most curtains, and the points along each.
pub(crate) const MAX_CURTAINS: usize = 6;
pub(crate) const CURTAIN_POINTS: usize = 128;
/// The footprint's radius about the viewer, km: the top of a curtain at
/// 400 km is above the horizon out to about 2260 km.
const FOOTPRINT_RADIUS: f32 = 2300.0;
/// The table's altitude range, km.
const TABLE_BOTTOM: f64 = 80.0;
const TABLE_TOP: f64 = 400.0;
/// The table's first row's characteristic energy (keV) and the step
/// between rows in octaves.
const TABLE_E0: f64 = 0.5;
const TABLE_OCTAVES: f64 = 0.67;
/// The soft electrons' share of the energy flux and their characteristic
/// energy (keV), beside the primaries ([`emission::emission_rows_of`]).
const SOFT_SHARE: f64 = 0.1;
/// The primaries' accelerating potential in units of the characteristic
/// energy, at keV energies ([`emission::emission_rows_of`]).
const ACCELERATION: f64 = 1.6;
const SOFT_E0: f64 = 0.25;
/// The strip's cells along and across the curtain.
const STRIP_X: usize = 256;
const STRIP_Y: usize = 32;
/// The strip's mip levels, 256 × 32 down to 1 × 1.
const STRIP_MIPS: u32 = 9;

/// An aurora over the scene. See the module documentation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aurora {
    /// How bright the display is (1: a bright display against a dark sky).
    pub intensity: f32,
    /// 0: quiet, smooth arcs; 0.5: folding, rayed curtains; 1: a substorm's
    /// breakup, curled and fragmented.
    pub activity: f32,
    /// How many arcs, one behind the other (1 to 6).
    pub curtains: u32,
    /// Where the arcs lie, degrees about +y from +x (the DayNight azimuth's
    /// convention).
    pub direction: f32,
    /// How far the first arc is from the world's origin over the ground,
    /// metres (0: overhead).
    pub distance: f32,
    /// The distance between successive arcs, metres.
    pub spread: f32,
    /// The precipitating electrons' characteristic energy, keV: soft
    /// (under 1) aurora are high, with red tops; hard (over 5) reach down
    /// with a pink-violet lower border.
    pub energy: f32,
    /// How fast the folds travel and the rays drift (1: as observed).
    pub speed: f32,
    pub seed: u32,
    /// The lines' colours (linear, their brightest channel 1): the oxygen
    /// green line, the oxygen red line and the nitrogen bands of the lower
    /// border. The defaults are the lines' spectral colours
    /// ([`Aurora::SPECTRAL`]); a camera's photograph renders them bluer.
    pub green: Vec3,
    pub red: Vec3,
    pub nitrogen: Vec3,
}

impl Default for Aurora {
    fn default() -> Self {
        Aurora {
            intensity: 1.0,
            activity: 0.4,
            curtains: 3,
            direction: 90.0,
            distance: 150_000.0,
            spread: 70_000.0,
            energy: 2.0,
            speed: 1.0,
            seed: 1,
            green: Aurora::SPECTRAL[0],
            red: Aurora::SPECTRAL[1],
            nitrogen: Aurora::SPECTRAL[2],
        }
    }
}

impl Aurora {
    /// The green line's, the red line's and the nitrogen bands' spectral
    /// colours ([`line_colors`], each scaled to its brightest channel).
    pub const SPECTRAL: [Vec3; 3] = [
        Vec3::new(0.29, 1.0, 0.0),
        Vec3::new(1.0, 0.0, 0.06),
        Vec3::new(0.57, 0.0, 1.0),
    ];
}

/// The overall scale from the table's units to the sky's radiance at
/// intensity 1.
const RADIANCE: f32 = 0.35;
/// The lines' column efficiencies, relative to the green line's, per unit
/// of deposited energy (Rees & Luckey 1974's ratios for electron aurora).
const EFFICIENCY_4278: f32 = 0.22;
const EFFICIENCY_1PG: f32 = 0.35;
const EFFICIENCY_6300: f32 = 5.0;

impl Aurora {
    /// The half-width of the curtains' sheet, km: wider as curls grow.
    fn half_width(&self) -> f32 {
        5.0 + 7.0 * self.activity.clamp(0.0, 1.0)
    }

    /// The frame uniform's block on a planet of `radius_km`: [0] the
    /// strength (1 when on), on, the radius, the footprint's radius; [1]
    /// the shell's bottom and top (km), the footprint's side, its rows
    /// with the table; [2..5] the prompt nitrogen, green and red lines'
    /// colours times the intensity, w: the march's least step (km), its
    /// largest, the conservative margin per texel.
    pub(crate) fn gpu(&self, radius_km: f32) -> [[f32; 4]; AURORA_VEC4S] {
        let k = self.intensity.max(0.0) * RADIANCE;
        // Each line's strength from physics, its hue as chosen.
        let [n2, green, red] = line_colors();
        let hue = |c: Vec3| c.max(Vec3::ZERO) / c.max_element().max(1e-6);
        let (n2, green, red) = (
            hue(self.nitrogen) * n2.max_element(),
            hue(self.green) * green.max_element(),
            hue(self.red) * red.max_element(),
        );
        [
            [1.0, 1.0, radius_km, FOOTPRINT_RADIUS],
            [
                TABLE_BOTTOM as f32,
                TABLE_TOP as f32,
                FOOTPRINT as f32,
                (FOOTPRINT + TABLE_ROWS) as f32,
            ],
            (n2 * k).extend(0.4).to_array(),
            (green * k).extend(60.0).to_array(),
            (red * k).extend(4.0 / FOOTPRINT as f32).to_array(),
        ]
    }

    /// The footprint pass's parameters at `seconds` of the clock, about a
    /// viewer at `viewer` (world x and z, km).
    pub(crate) fn footprint(&self, seconds: f64, viewer: Vec2) -> FootprintParams {
        let mut p: FootprintParams = bytemuck::Zeroable::zeroed();
        let activity = self.activity.clamp(0.0, 1.0);
        let curtains = self.curtains.clamp(1, MAX_CURTAINS as u32) as usize;
        let hw = self.half_width();
        let red_sigma = 14.0_f32;
        // The diffuse red flux carries the same column as the sheet: the
        // sheet's dye, a Gaussian 1.5 cells wide of 32 across 2 hw.
        let core_column = 1.5 * (2.0 * hw / STRIP_Y as f32) * std::f32::consts::PI.sqrt();
        let red_scale = core_column / (red_sigma * (2.0 * std::f32::consts::PI).sqrt());
        p.head = [
            FOOTPRINT as f32,
            FOOTPRINT_RADIUS,
            curtains as f32,
            CURTAIN_POINTS as f32,
        ];
        p.look = [activity * 3.0, hw, red_sigma, red_scale];
        let t = seconds * f64::from(self.speed.max(0.0));
        let toward = Quat::from_rotation_y(self.direction.to_radians()) * Vec3::X;
        let toward = glam::DVec2::new(f64::from(toward.x), f64::from(toward.z));
        let side = glam::DVec2::new(-toward.y, toward.x);
        let viewer = glam::DVec2::new(f64::from(viewer.x), f64::from(viewer.y));
        let mut rng = SplitMix(u64::from(self.seed).wrapping_mul(0x9e37_79b9) ^ 0x0a17_02a5);
        // The oval's curvature (km) and the folds' wavelengths (km), speeds
        // (km/s) and amplitudes (km, at full activity).
        let oval = 2600.0;
        let folds = [
            (1100.0, 0.9, 60.0),
            (520.0, 1.6, 30.0),
            (230.0, 2.4, 14.0),
            (110.0, 3.5, 6.0),
        ];
        let amp = 0.15 + 1.1 * f64::from(activity);
        // The strip's period along the curtain keeps its cells square.
        let period = f64::from(hw) * 2.0 * (STRIP_X / STRIP_Y) as f64;
        let span = 2.0 * f64::from(FOOTPRINT_RADIUS) + 600.0;
        for c in 0..curtains {
            let d0 = (f64::from(self.distance.max(0.0))
                + c as f64 * f64::from(self.spread.max(0.0)))
                * 0.001;
            let centre = toward * (d0 + oval);
            let phases: Vec<f64> = (0..folds.len())
                .map(|_| rng.unit() * std::f64::consts::TAU)
                .collect();
            let direction = if rng.unit() < 0.5 { -1.0 } else { 1.0 };
            let drift = (0.6 + 0.8 * rng.unit()) * (0.2 + f64::from(activity)) * 3.0 * direction;
            let offset = rng.unit();
            let energy = f64::from(self.energy.max(0.1)) * (0.8 + 0.5 * rng.unit());
            let bright = 1.0 / (1.0 + 0.45 * c as f64);
            let pulse = (40.0 + 50.0 * rng.unit(), rng.unit() * std::f64::consts::TAU);
            let noise_seed = rng.next();
            // Rays: none on a quiet arc, most on an active one, fewer
            // where the curls of a breakup take over.
            let a = f64::from(activity);
            let rays = 0.65 * smooth(0.0, 0.15, a) * (1.0 - 0.6 * smooth(0.5, 1.0, a));
            p.curtain[c] = [
                energy as f32,
                (noise_seed & 0xffff) as f32,
                rays as f32,
                0.0,
            ];
            // The strip's shift along the curve (drift wraps by whole periods).
            let shift = (offset * period - drift * t).rem_euclid(period);
            let mut u = 0.0;
            let mut last: Option<glam::DVec2> = None;
            for j in 0..CURTAIN_POINTS {
                let s = j as f64 / (CURTAIN_POINTS - 1) as f64 * 2.0 - 1.0;
                // Finer near the middle of the arc, where it is nearest.
                let a = span * 0.5 * (0.35 * s + 0.65 * s * s * s);
                let phi = a / oval;
                let normal = -toward * phi.cos() + side * phi.sin();
                let mut y = 0.0;
                for (k, &(lambda, speed, height)) in folds.iter().enumerate() {
                    let w = std::f64::consts::TAU / lambda;
                    y += height * amp * (w * a + phases[k] - w * speed * t).sin();
                }
                let at = centre + normal * (oval + y);
                u += last.map_or(0.0, |l| (at - l).length());
                last = Some(at);
                let local = at - viewer;
                // Brightness along the arc: slow patches, a slow pulsation,
                // the ends faded.
                let patch = value_noise(noise_seed, a / 300.0 + t * 0.004)
                    * (0.55 + 0.45 * value_noise(noise_seed ^ 0x5bd1, a / 45.0 - t * 0.02));
                let breathe = 0.8 + 0.2 * (std::f64::consts::TAU * t / pulse.0 + pulse.1).sin();
                let ends = 1.0 - smooth(0.85, 1.0, s.abs());
                let w = bright * (0.35 + 0.65 * patch) * breathe * ends;
                // The sheet's flow varies along the arc: its curls and rays
                // bunch and spread (a slow warp of the strip's coordinate).
                let warp =
                    (value_noise(noise_seed ^ 0x77, a / 240.0 + t * 0.003) - 0.5) * 0.7 * period;
                let strip = (u + shift + warp) / period;
                p.points[c * CURTAIN_POINTS + j] =
                    [local.x as f32, local.y as f32, strip as f32, w as f32];
            }
        }
        p
    }
}

/// The footprint pass's uniform (`aurora_footprint.wgsl` `Params`).
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct FootprintParams {
    /// x: side, y: radius (km), z: curtains, w: points per curtain.
    pub head: [f32; 4],
    /// x: the strip's snapshot (activity × 3), y: the sheet's half-width
    /// (km), z: the red line's spread (km), w: its flux's scale.
    pub look: [f32; 4],
    /// Per curtain: x its characteristic energy (keV).
    pub curtain: [[f32; 4]; MAX_CURTAINS],
    /// Per point: xz about the viewer (km), the strip's coordinate along
    /// (periods), the brightness.
    pub points: [[f32; 4]; MAX_CURTAINS * CURTAIN_POINTS],
}

fn smooth(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

struct SplitMix(u64);

impl SplitMix {
    fn next(&mut self) -> u64 {
        og_core::hash::splitmix64_next(&mut self.0)
    }
    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Smooth 1D value noise in 0..1 of two octaves.
fn value_noise(seed: u64, x: f64) -> f64 {
    let lattice = |i: i64| SplitMix(seed ^ (i as u64).wrapping_mul(0x2545_f491_4f6c_dd1d)).unit();
    let octave = |x: f64| {
        let i = x.floor();
        let f = x - i;
        let f = f * f * (3.0 - 2.0 * f);
        lattice(i as i64) * (1.0 - f) + lattice(i as i64 + 1) * f
    };
    (octave(x) * 0.7 + octave(x * 2.7 + 17.0) * 0.3).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests;
