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
/// energy (keV), beside the primaries ([`emission_rows_of`]).
const SOFT_SHARE: f64 = 0.1;
/// The primaries' accelerating potential in units of the characteristic
/// energy, at keV energies ([`emission_rows_of`]).
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
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
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

// ── the deposition table ──────────────────────────────────────────────

/// The U.S. Standard Atmosphere 1976's mass density (km, kg/m³).
const US76: [(f64, f64); 23] = [
    (80.0, 1.846e-5),
    (85.0, 6.958e-6),
    (90.0, 3.416e-6),
    (95.0, 1.393e-6),
    (100.0, 5.604e-7),
    (105.0, 2.325e-7),
    (110.0, 9.708e-8),
    (115.0, 4.289e-8),
    (120.0, 2.222e-8),
    (130.0, 8.152e-9),
    (140.0, 3.831e-9),
    (150.0, 2.076e-9),
    (160.0, 1.233e-9),
    (170.0, 7.815e-10),
    (180.0, 5.194e-10),
    (190.0, 3.581e-10),
    (200.0, 2.541e-10),
    (220.0, 1.367e-10),
    (250.0, 6.073e-11),
    (300.0, 1.916e-11),
    (350.0, 7.014e-12),
    (400.0, 2.803e-12),
    (500.0, 5.215e-13),
];

/// The density at `z` km (kg/m³), log-linear between the table's heights.
fn density(z: f64) -> f64 {
    let k = US76
        .windows(2)
        .position(|w| z < w[1].0)
        .unwrap_or(US76.len() - 2);
    let (a, b) = (US76[k], US76[k + 1]);
    let scale = (b.0 - a.0) / (a.1 / b.1).ln();
    a.1 * (-(z - a.0) / scale).exp()
}

/// The mass of air above `z` km (kg/m²), each interval integrated as the
/// exponential it is, the last one's scale height above the table.
fn shielding(z: f64) -> f64 {
    let mut m = 0.0;
    for w in US76.windows(2) {
        let (a, b) = (w[0], w[1]);
        if z >= b.0 {
            continue;
        }
        let scale = (b.0 - a.0) / (a.1 / b.1).ln() * 1000.0;
        let from = z.max(a.0);
        m += scale * (density(from) - b.1);
    }
    let (a, b) = (US76[US76.len() - 2], US76[US76.len() - 1]);
    let top = (b.0 - a.0) / (a.1 / b.1).ln() * 1000.0;
    m + top * density(z.max(b.0))
}

/// Lazarev's energy deposition (keV per km per electron) of electrons of
/// `e` keV where the air's density is `d` g/cm³ under `m` g/cm² of air
/// (Lawlor & Genetti 2011, section 2.1).
fn lazarev(d: f64, m: f64, e: f64) -> f64 {
    let me = 4.6e-6 * e.powf(1.65);
    let r = m / me;
    let l = 4.2 * r * (-r * r - r).exp() + 0.48 * (-17.4 * r.powf(1.37)).exp();
    l * e * d / me * 1e5
}

/// The fraction of the air that is atomic oxygen at `z` km (MSIS-like:
/// a few percent at 100 km, most of it above 200 km).
fn oxygen(z: f64) -> f64 {
    0.9 / (1.0 + (-(z - 160.0) / 30.0).exp())
}

/// The spectrum's energies (keV, 0.1 to 40, log-spaced) and each one's
/// width.
fn energies() -> impl Iterator<Item = (f64, f64)> {
    (0..48).map(|k| {
        let e = 0.1 * 400f64.powf(k as f64 / 47.0);
        (e, e * 400f64.ln() / 47.0)
    })
}

/// The volume emission rates (per km, per unit energy flux into the
/// column) at `n` heights for characteristic energy `e0`, of a Maxwellian
/// spectrum's deposition: the prompt nitrogen bands (427.8 nm and the
/// first positive band), OI 557.7 nm and OI 630.0 nm, before their
/// efficiencies. `deposits[z][k]`: Lazarev's deposition at height z of
/// the k-th energy of [`energies`].
fn emission_rows_of(e0: f64, heights: &[f64], deposits: &[Vec<f64>]) -> Vec<[f64; 3]> {
    let dz = (TABLE_TOP - TABLE_BOTTOM) / heights.len() as f64;
    // The spectrum: the primaries and, with a tenth of
    // the energy flux, the soft electrons that come down with every
    // discrete arc (secondaries and degraded primaries returning from the
    // ionosphere below, trapped and reflected above the accelerating
    // region: Evans, J. Geophys. Res. 79, 1974), a Maxwellian of 0.25 keV.
    // They deposit high, where the red line glows: an arc's top fades from
    // green into a faint red even when its primaries are hard.
    //
    // A discrete arc's primaries have fallen through a field-aligned
    // potential (the inverted V: Evans 1974): an accelerated Maxwellian,
    // nothing below the potential V and a source temperature kT above it
    // (Strickland, Meier, Hecht & Christensen, J. Geophys. Res. 94, 1989).
    // At keV energies V = 1.6 e0 and kT = 0.2 e0, whose mean energy is a
    // Maxwellian of e0's (2 e0), so `energy` keeps its meaning; lacking
    // the Maxwellian's high tail, the primaries stop within a few
    // kilometres of one depth, the sharp, bright lower edge of an arc, the
    // light fading upward. Soft precipitation (diffuse aurora, the cusp)
    // is not accelerated: below 1.5 keV the potential fades out, the mean
    // energy kept, to a plain Maxwellian at 0.5 keV.
    let maxwellian = |e: f64, k: f64| e * (-e / k).exp();
    let a = smooth(0.5, 1.5, e0);
    let (v, kt) = (ACCELERATION * a * e0, (1.0 - ACCELERATION * a / 2.0) * e0);
    let accelerated = |e: f64| {
        if e >= v {
            e * (-(e - v) / kt).exp()
        } else {
            0.0
        }
    };
    let flux = |f: &dyn Fn(f64) -> f64| -> f64 { energies().map(|(e, de)| f(e) * e * de).sum() };
    let (primary, soft) = (flux(&accelerated), flux(&|e| maxwellian(e, SOFT_E0)));
    let deposit: Vec<f64> = deposits
        .iter()
        .map(|row| {
            energies()
                .zip(row)
                .map(|((e, de), a)| {
                    let n = (1.0 - SOFT_SHARE) * accelerated(e) / primary
                        + SOFT_SHARE * maxwellian(e, SOFT_E0) / soft;
                    n * a * de
                })
                .sum()
        })
        .collect();
    let total: f64 = deposit.iter().sum::<f64>() * dz;
    let n2_ref = 1.0 - oxygen(100.0);
    heights
        .iter()
        .zip(&deposit)
        .map(|(&z, &a)| {
            let a = a / total;
            let rho = density(z);
            // Quenching by collisions: the green line's 0.7 s state below
            // about 97 km, the red line's 110 s state below about 210 km.
            let green = a / (1.0 + rho / 1.2e-6);
            let red = a * oxygen(z) / (1.0 + rho / 1.5e-10);
            let n2 = a * (1.0 - oxygen(z)) / n2_ref;
            [n2, green, red]
        })
        .collect()
}

/// The table's heights (km, texel centres) and Lazarev's deposition at
/// each for each energy.
fn deposits(n: usize) -> (Vec<f64>, Vec<Vec<f64>>) {
    let dz = (TABLE_TOP - TABLE_BOTTOM) / n as f64;
    let heights: Vec<f64> = (0..n)
        .map(|x| TABLE_BOTTOM + (x as f64 + 0.5) * dz)
        .collect();
    let deposits = heights
        .iter()
        .map(|&z| {
            let d = density(z) * 1e-3; // g/cm³
            let m = shielding(z) * 0.1; // g/cm²
            energies().map(|(e, _)| lazarev(d, m, e)).collect()
        })
        .collect();
    (heights, deposits)
}

#[cfg(test)]
fn emission_rows(e0: f64, n: usize) -> Vec<[f64; 3]> {
    let (heights, deposits) = deposits(n);
    emission_rows_of(e0, &heights, &deposits)
}

/// The table: `TABLE_ROWS` rows of `n` texels (rgba: prompt N₂, 557.7,
/// 630.0, 0), each row's characteristic energy `TABLE_E0` times
/// 2^(row × `TABLE_OCTAVES`).
pub(crate) fn table(n: usize) -> Vec<[f32; 4]> {
    let (heights, deposits) = deposits(n);
    (0..TABLE_ROWS)
        .flat_map(|row| {
            let e0 = TABLE_E0 * 2f64.powf(f64::from(row) * TABLE_OCTAVES);
            emission_rows_of(e0, &heights, &deposits)
                .into_iter()
                .map(|[a, b, c]| [a as f32, b as f32, c as f32, 0.0])
        })
        .collect()
}

/// The lines' colours in linear sRGB per photon (radiant power falls as
/// 1/λ), weighted by their efficiencies: the prompt nitrogen bands, the
/// green line, the red line.
fn line_colors() -> [Vec3; 3] {
    // CIE 1931 2° colour matching values (x̄, ȳ, z̄) at the line, linearly
    // interpolated from the 5 nm table.
    let line = |xyz: [f32; 3], nm: f32| {
        let [x, y, z] = xyz;
        let rgb = Vec3::new(
            3.2406 * x - 1.5372 * y - 0.4986 * z,
            -0.9689 * x + 1.8758 * y + 0.0415 * z,
            0.0557 * x - 0.2040 * y + 1.0570 * z,
        );
        // Into the gamut by adding white (desaturating, keeping the hue).
        let lift = (-rgb.min_element()).max(0.0);
        (rgb + Vec3::splat(lift)) * (557.7 / nm)
    };
    let blue = line([0.2535, 0.0097, 1.233], 427.8);
    let first_positive = line([0.1649, 0.0610, 0.0], 660.0);
    let green = line([0.5566, 0.9973, 0.0047], 557.7);
    let red = line([0.6424, 0.2650, 0.0], 630.0);
    [
        blue * EFFICIENCY_4278 + first_positive * EFFICIENCY_1PG,
        green,
        red * EFFICIENCY_6300,
    ]
}

// ── the Kelvin–Helmholtz strip ───────────────────────────────────────

/// The sheet's electron flux (the paper's fluid dye) at four moments of a
/// Kelvin–Helmholtz roll-up, one per channel (r: the smooth sheet, a: the
/// turbulent one), `STRIP_X` along (periodic) by `STRIP_Y` across.
///
/// The paper's 2D incompressible flow (Stam 1999) in its vorticity form:
/// the vorticity and the dye are carried by the flow (semi-Lagrangian,
/// with back-and-forth error compensation, Kim et al. 2005, so the thin
/// sheet is not smeared), and the flow is the curl of the stream function
/// solved exactly each step (a Fourier transform along the periodic strip,
/// a tridiagonal solve across it, walls at its edges), in place of the
/// paper's multigrid pressure projection.
pub(crate) fn kh_strip(seed: u32) -> Vec<[u8; 4]> {
    let (nx, ny) = (STRIP_X, STRIP_Y);
    let mut rng = SplitMix(u64::from(seed) ^ 0x6b68_5f73_7472_6970);
    let yc = ny as f32 * 0.5;
    // The sheet, displaced by a spectrum of small waves along it, and the
    // shear across it (velocity -1 to 1 over 1.5 cells).
    let modes: Vec<(f32, f32, f32)> = (2..28)
        .map(|m| {
            (
                m as f32,
                rng.unit() as f32 * std::f32::consts::TAU,
                0.3 + 0.7 * rng.unit() as f32,
            )
        })
        .collect();
    let shear = 1.5f32;
    let mut vort = vec![0.0f32; nx * ny];
    let mut dye = vec![0.0f32; nx * ny];
    for x in 0..nx {
        let mut eta = 0.0;
        for &(m, phase, a) in &modes {
            eta += a * (std::f32::consts::TAU * m * (x as f32 + 0.5) / nx as f32 + phase).sin()
                / m.sqrt();
        }
        eta *= 0.12;
        for y in 0..ny {
            let dy = (y as f32 + 0.5 - yc - eta) / shear;
            let sech = 1.0 / dy.cosh();
            vort[y * nx + x] = -sech * sech / shear;
            dye[y * nx + x] = (-dy * dy).exp();
        }
    }
    let dt = 0.6f32;
    let snapshots = [0usize, 45, 80, 140];
    let mut out = vec![[0u8; 4]; nx * ny];
    let mut step = 0;
    let mut flow = (vec![0.0f32; nx * ny], vec![0.0f32; nx * ny]);
    for (channel, &until) in snapshots.iter().enumerate() {
        while step < until {
            stream_flow(&vort, nx, ny, &mut flow);
            vort = advect_bfecc(&vort, &flow, nx, ny, dt);
            dye = advect_bfecc(&dye, &flow, nx, ny, dt);
            step += 1;
        }
        let peak = dye.iter().fold(1e-6f32, |m, &d| m.max(d));
        for (o, &d) in out.iter_mut().zip(&dye) {
            o[channel] = ((d / peak).clamp(0.0, 1.0) * 255.0).round() as u8;
        }
    }
    out
}

/// The strip's mip levels (width, height, texels), 2×2 box averages down
/// to one texel, the first the strip itself. The footprint samples the
/// level of its texel's size: the sheet is far thinner than a footprint
/// texel away from the viewer (half a kilometre against a few), and read
/// at one point per texel it shows only where it crosses a texel's
/// centre, an even comb of false rays. The averages keep each texel's
/// flux, the sheet's column, at every distance.
fn strip_mips(strip: &[[u8; 4]]) -> Vec<(usize, usize, Vec<[u8; 4]>)> {
    let mut levels = vec![(STRIP_X, STRIP_Y, strip.to_vec())];
    while levels.len() < STRIP_MIPS as usize {
        let (w, h, last) = levels.last().expect("the strip itself");
        let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
        let mut next = vec![[0u8; 4]; nw * nh];
        for y in 0..nh {
            for x in 0..nw {
                let mut sum = [0u32; 4];
                let (ys, xs) = ((h / nh).max(1), w / nw);
                for dy in 0..ys {
                    for dx in 0..xs {
                        let t = last[(y * ys + dy) * w + x * xs + dx];
                        for c in 0..4 {
                            sum[c] += u32::from(t[c]);
                        }
                    }
                }
                let n = (ys * xs) as u32;
                next[y * nw + x] = sum.map(|v| ((v + n / 2) / n) as u8);
            }
        }
        levels.push((nw, nh, next));
    }
    levels
}

/// Bilinear sample of a strip field, periodic along x, clamped across y.
fn strip_sample(f: &[f32], nx: usize, ny: usize, x: f32, y: f32) -> f32 {
    let y = y.clamp(0.0, ny as f32 - 1.0);
    let x = x.rem_euclid(nx as f32);
    let (x0, y0) = (x.floor() as usize % nx, y.floor() as usize);
    let (x1, y1) = ((x0 + 1) % nx, (y0 + 1).min(ny - 1));
    let (fx, fy) = (x - x.floor(), y - y.floor());
    let a = f[y0 * nx + x0] * (1.0 - fx) + f[y0 * nx + x1] * fx;
    let b = f[y1 * nx + x0] * (1.0 - fx) + f[y1 * nx + x1] * fx;
    a * (1.0 - fy) + b * fy
}

/// `f` carried by `flow` for `dt` (back-and-forth error compensation and
/// correction, the result limited to its source's neighbourhood).
fn advect_bfecc(f: &[f32], flow: &(Vec<f32>, Vec<f32>), nx: usize, ny: usize, dt: f32) -> Vec<f32> {
    let (u, v) = flow;
    let carry = |src: &[f32], sign: f32| -> Vec<f32> {
        (0..nx * ny)
            .map(|i| {
                let (x, y) = ((i % nx) as f32, (i / nx) as f32);
                strip_sample(src, nx, ny, x - sign * dt * u[i], y - sign * dt * v[i])
            })
            .collect()
    };
    let forward = carry(f, 1.0);
    let back = carry(&forward, -1.0);
    let corrected: Vec<f32> = f
        .iter()
        .zip(&back)
        .map(|(a, b)| a + 0.5 * (a - b))
        .collect();
    let mut result = carry(&corrected, 1.0);
    // Clamp to the values the plain step blended (no new extremes).
    for (i, r) in result.iter_mut().enumerate() {
        let (x, y) = ((i % nx) as f32 - dt * u[i], (i / nx) as f32 - dt * v[i]);
        let y = y.clamp(0.0, ny as f32 - 1.0);
        let (x0, y0) = (
            x.rem_euclid(nx as f32).floor() as usize % nx,
            y.floor() as usize,
        );
        let (x1, y1) = ((x0 + 1) % nx, (y0 + 1).min(ny - 1));
        let around = [
            f[y0 * nx + x0],
            f[y0 * nx + x1],
            f[y1 * nx + x0],
            f[y1 * nx + x1],
        ];
        let lo = around.iter().fold(f32::MAX, |m, &a| m.min(a));
        let hi = around.iter().fold(f32::MIN, |m, &a| m.max(a));
        *r = r.clamp(lo, hi);
    }
    result
}

/// The flow (u, v) of vorticity `vort`: the stream function ψ solves
/// ∇²ψ = -ω with ψ = 0 on the walls, by a Fourier transform along x and a
/// tridiagonal solve across y per wavenumber; u = ∂ψ/∂y, v = -∂ψ/∂x.
fn stream_flow(vort: &[f32], nx: usize, ny: usize, flow: &mut (Vec<f32>, Vec<f32>)) {
    // Rows to the frequency domain.
    let mut spectrum: Vec<Vec<(f32, f32)>> = (0..ny)
        .map(|y| {
            let mut row: Vec<(f32, f32)> = vort[y * nx..(y + 1) * nx]
                .iter()
                .map(|&w| (-w, 0.0))
                .collect();
            fft(&mut row, false);
            row
        })
        .collect();
    for k in 0..nx {
        // (ψ[y-1] - (2 + λ) ψ[y] + ψ[y+1]) = rhs[y], λ the discrete kx².
        let lambda = 2.0 - 2.0 * (std::f32::consts::TAU * k as f32 / nx as f32).cos();
        let diag = -(2.0 + lambda);
        let mut c = vec![0.0f32; ny];
        let mut d = vec![(0.0f32, 0.0f32); ny];
        for (y, row) in spectrum.iter().enumerate() {
            let rhs = row[k];
            let m = diag - if y > 0 { c[y - 1] } else { 0.0 };
            c[y] = 1.0 / m;
            let prev = if y > 0 { d[y - 1] } else { (0.0, 0.0) };
            d[y] = ((rhs.0 - prev.0) / m, (rhs.1 - prev.1) / m);
        }
        for y in (0..ny - 1).rev() {
            d[y] = (d[y].0 - c[y] * d[y + 1].0, d[y].1 - c[y] * d[y + 1].1);
        }
        for (row, v) in spectrum.iter_mut().zip(&d) {
            row[k] = *v;
        }
    }
    let psi: Vec<f32> = spectrum
        .iter_mut()
        .flat_map(|row| {
            fft(row, true);
            row.iter()
                .map(|&(re, _)| re / nx as f32)
                .collect::<Vec<f32>>()
        })
        .collect();
    let at = |x: usize, y: isize| -> f32 {
        if y < 0 || y >= ny as isize {
            0.0
        } else {
            psi[y as usize * nx + x]
        }
    };
    for y in 0..ny {
        for x in 0..nx {
            let (xl, xr) = ((x + nx - 1) % nx, (x + 1) % nx);
            let yi = y as isize;
            flow.0[y * nx + x] = 0.5 * (at(x, yi + 1) - at(x, yi - 1));
            flow.1[y * nx + x] = -0.5 * (at(xr, yi) - at(xl, yi));
        }
    }
}

/// An in-place radix-2 complex FFT (`inverse`: unscaled inverse).
fn fft(a: &mut [(f32, f32)], inverse: bool) {
    let n = a.len();
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            a.swap(i, j);
        }
    }
    let mut len = 2;
    while len <= n {
        let angle = std::f32::consts::TAU / len as f32 * if inverse { 1.0 } else { -1.0 };
        for start in (0..n).step_by(len) {
            for k in 0..len / 2 {
                let (s, c) = (angle * k as f32).sin_cos();
                let (ur, ui) = a[start + k];
                let (vr0, vi0) = a[start + k + len / 2];
                let (vr, vi) = (vr0 * c - vi0 * s, vr0 * s + vi0 * c);
                a[start + k] = (ur + vr, ui + vi);
                a[start + k + len / 2] = (ur - vr, ui - vi);
            }
        }
        len <<= 1;
    }
}

// ── the GPU side ─────────────────────────────────────────────────────

/// The footprint and table texture the sky pass reads (its group's
/// binding 6), and what draws it: a 1×1 placeholder until a frame has an
/// aurora.
pub(crate) struct AuroraMap {
    pub view: wgpu::TextureView,
    texture: wgpu::Texture,
    pass: Option<FootprintPass>,
}

struct FootprintPass {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
    uniform: wgpu::Buffer,
    sampler: wgpu::Sampler,
    strip: wgpu::TextureView,
    seed: u32,
    group: wgpu::BindGroup,
}

fn map_texture(device: &wgpu::Device, width: u32, height: u32, storage: bool) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("aurora footprint"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: if storage {
            wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_DST
        } else {
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST
        },
        view_formats: &[],
    })
}

fn write_rows(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    y: u32,
    width: u32,
    texels: &[[f32; 4]],
) {
    let bytes: Vec<u8> = texels
        .iter()
        .flat_map(|t| t.map(crate::lightmap::f16_bits))
        .flat_map(u16::to_le_bytes)
        .collect();
    let rows = texels.len() as u32 / width;
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d { x: 0, y, z: 0 },
            aspect: wgpu::TextureAspect::All,
        },
        &bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width * 8),
            rows_per_image: Some(rows),
        },
        wgpu::Extent3d {
            width,
            height: rows,
            depth_or_array_layers: 1,
        },
    );
}

impl AuroraMap {
    pub(crate) fn placeholder(device: &wgpu::Device, queue: &wgpu::Queue) -> AuroraMap {
        let texture = map_texture(device, 1, 1, false);
        write_rows(queue, &texture, 0, 1, &[[0.0; 4]]);
        AuroraMap {
            view: texture.create_view(&Default::default()),
            texture,
            pass: None,
        }
    }

    /// Whether the full texture exists (the sky group must be made again
    /// the frame it does).
    pub(crate) fn made(&self) -> bool {
        self.pass.is_some()
    }

    /// Makes the footprint texture, its table and the pass that draws it
    /// (once), and the strip again when the seed changed.
    fn ensure(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, seed: u32) {
        if self.pass.as_ref().is_some_and(|p| p.seed == seed) {
            return;
        }
        let strip_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("aurora strip"),
            size: wgpu::Extent3d {
                width: STRIP_X as u32,
                height: STRIP_Y as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: STRIP_MIPS,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        for (level, (w, h, texels)) in strip_mips(&kh_strip(seed)).iter().enumerate() {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &strip_texture,
                    mip_level: level as u32,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                bytemuck::cast_slice(texels),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(*w as u32 * 4),
                    rows_per_image: Some(*h as u32),
                },
                wgpu::Extent3d {
                    width: *w as u32,
                    height: *h as u32,
                    depth_or_array_layers: 1,
                },
            );
        }
        let strip = strip_texture.create_view(&Default::default());
        if let Some(pass) = &mut self.pass {
            pass.group = footprint_group(
                device,
                &pass.layout,
                &pass.uniform,
                &strip,
                &pass.sampler,
                &self.view,
            );
            pass.strip = strip;
            pass.seed = seed;
            return;
        }
        let texture = map_texture(device, FOOTPRINT, FOOTPRINT + TABLE_ROWS, true);
        write_rows(
            queue,
            &texture,
            FOOTPRINT,
            FOOTPRINT,
            &table(FOOTPRINT as usize),
        );
        let view = texture.create_view(&Default::default());
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("aurora footprint"),
            source: wgpu::ShaderSource::Wgsl(FOOTPRINT_WGSL.into()),
        });
        let entry = |binding, ty| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty,
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("aurora footprint"),
            entries: &[
                entry(
                    0,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                ),
                entry(
                    1,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                ),
                entry(
                    2,
                    wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                ),
                entry(
                    3,
                    wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba16Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                ),
            ],
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("aurora footprint"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("aurora footprint"),
                    bind_group_layouts: &[Some(&layout)],
                    immediate_size: 0,
                }),
            ),
            module: &module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("aurora footprint"),
            size: std::mem::size_of::<FootprintParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // Along the curtain the strip repeats; across it, it ends.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("aurora strip"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            anisotropy_clamp: 8,
            ..Default::default()
        });
        let group = footprint_group(device, &layout, &uniform, &strip, &sampler, &view);
        self.pass = Some(FootprintPass {
            pipeline,
            layout,
            uniform,
            sampler,
            strip,
            seed,
            group,
        });
        self.view = view;
        self.texture = texture;
    }

    /// Draws the footprint for `aurora` at `seconds` about the viewer at
    /// `viewer` (world x and z, metres): its own submission, before the
    /// frame's, so every pass of the frame reads it.
    pub(crate) fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        aurora: &Aurora,
        seconds: f64,
        viewer: Vec2,
    ) {
        self.ensure(device, queue, aurora.seed);
        let pass = self.pass.as_ref().expect("made above");
        let params = aurora.footprint(seconds, viewer * 0.001);
        queue.write_buffer(&pass.uniform, 0, bytemuck::bytes_of(&params));
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("aurora footprint"),
        });
        {
            let mut compute = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("aurora footprint"),
                timestamp_writes: None,
            });
            compute.set_pipeline(&pass.pipeline);
            compute.set_bind_group(0, &pass.group, &[]);
            compute.dispatch_workgroups(FOOTPRINT / 8, FOOTPRINT / 8, 1);
        }
        queue.submit(Some(encoder.finish()));
    }
}

fn footprint_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    uniform: &wgpu::Buffer,
    strip: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
    out: &wgpu::TextureView,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("aurora footprint"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(strip),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(out),
            },
        ],
    })
}

/// The footprint pass's compute shader.
pub(crate) const FOOTPRINT_WGSL: &str = wgsl!("aurora_footprint.wgsl");

#[cfg(test)]
mod tests {
    use super::*;

    fn peak(e0: f64, line: usize) -> f64 {
        let n = 640;
        let rows = emission_rows(e0, n);
        let k = (0..n)
            .max_by(|&a, &b| rows[a][line].total_cmp(&rows[b][line]))
            .unwrap();
        TABLE_BOTTOM + (k as f64 + 0.5) * (TABLE_TOP - TABLE_BOTTOM) / n as f64
    }

    fn column(e0: f64, line: usize) -> f64 {
        let n = 640;
        let dz = (TABLE_TOP - TABLE_BOTTOM) / n as f64;
        emission_rows(e0, n).iter().map(|r| r[line] * dz).sum()
    }

    /// The heights the lines are seen at (Whiter et al. 2023): for typical
    /// precipitation the green and the blue lines peak near 114 km, harder
    /// electrons reach lower, and the red line glows above 200 km in soft
    /// aurora, much brighter against the green there than in hard aurora.
    #[test]
    fn the_lines_peak_where_they_are_observed() {
        let (green, blue) = (peak(2.0, 1), peak(2.0, 0));
        assert!((105.0..125.0).contains(&green), "green {green}");
        assert!((blue - green).abs() < 6.0, "blue {blue} green {green}");
        assert!(peak(10.0, 1) < green - 5.0, "hard {}", peak(10.0, 1));
        let red = peak(0.5, 2);
        assert!(red > 195.0, "red {red}");
        // The soft electrons that come with every arc keep a little red
        // over hard aurora too (Evans 1974), a fifth of the green's light
        // at most at 8 keV (the 630.0 nm line's efficiency counted).
        let soft = column(0.5, 2) / column(0.5, 1);
        let hard = column(8.0, 2) / column(8.0, 1);
        assert!(soft > 3.0 * hard, "red/green soft {soft} hard {hard}");
        assert!(hard * f64::from(EFFICIENCY_6300) < 0.25, "hard {hard}");
        // Below 95 km the green is quenched and the nitrogen bands (the
        // pink-violet lower border) outshine it relative to the peak.
        let rows = emission_rows(10.0, 640);
        let at = |z: f64| ((z - TABLE_BOTTOM) / (TABLE_TOP - TABLE_BOTTOM) * 640.0) as usize;
        let low = rows[at(90.0)];
        let mid = rows[at(115.0)];
        assert!(low[0] / low[1] > 1.5 * mid[0] / mid[1], "{low:?} {mid:?}");
    }

    /// An arc's lower edge is its sharpest: below the green peak the light
    /// rises from a tenth to full within 10 km, and above it takes over
    /// twice as far to fall to a fifth (the accelerated primaries stop
    /// within a few kilometres of one depth).
    #[test]
    fn the_arcs_lower_edge_is_sharp() {
        let n = 640;
        let rows = emission_rows(2.0, n);
        let height =
            |k: usize| TABLE_BOTTOM + (k as f64 + 0.5) * (TABLE_TOP - TABLE_BOTTOM) / n as f64;
        let g = rows.iter().map(|r| r[1]).fold(0.0, f64::max);
        let k = rows.iter().position(|r| r[1] == g).expect("a peak");
        let below = (0..k)
            .rev()
            .find(|&j| rows[j][1] < 0.1 * g)
            .expect("an edge");
        let above = (k..n).find(|&j| rows[j][1] < 0.2 * g).expect("a top");
        let (rise, fall) = (height(k) - height(below), height(above) - height(k));
        assert!(rise < 10.0, "rise {rise} km");
        assert!(fall > 2.0 * rise, "rise {rise} km fall {fall} km");
    }

    /// The lines' colours: the green line green, the red one red, the
    /// nitrogen bands violet to magenta (blue and red over green).
    #[test]
    fn the_lines_have_their_colours() {
        let [n2, green, red] = line_colors();
        assert!(
            green.y > 2.0 * green.x && green.y > 4.0 * green.z.max(0.01),
            "{green}"
        );
        assert!(red.x > 5.0 * red.y.max(0.01), "{red}");
        assert!(n2.z > n2.y && n2.x > n2.y, "{n2}");
    }

    /// The default colours are the lines' own.
    #[test]
    fn the_spectral_defaults_are_the_lines_colours() {
        let [n2, green, red] = line_colors();
        for (c, d) in [green, red, n2].iter().zip(Aurora::SPECTRAL) {
            let c = *c / c.max_element();
            assert!((c - d).abs().max_element() < 0.01, "{c} vs {d}");
        }
    }

    /// The sheet starts smooth and rolls up into curls: the later
    /// snapshots vary more along the curtain than the first.
    #[test]
    fn the_strip_rolls_up() {
        let strip = kh_strip(1);
        let variation = |c: usize| {
            let row = STRIP_Y / 2 + 3;
            let v: Vec<f32> = (0..STRIP_X)
                .map(|x| f32::from(strip[row * STRIP_X + x][c]))
                .collect();
            let mean = v.iter().sum::<f32>() / v.len() as f32;
            v.iter().map(|x| (x - mean).abs()).sum::<f32>() / v.len() as f32
        };
        assert!(
            variation(2) > 4.0 * variation(0).max(1.0),
            "{} {}",
            variation(0),
            variation(2)
        );
        assert_eq!(strip, kh_strip(1), "deterministic");
    }

    /// The curtains are a function of the clock: the same at the same
    /// second, the folds moved a second later, and the strip's coordinate
    /// continuous along each curve.
    #[test]
    fn the_curtains_follow_the_clock() {
        let a = Aurora::default();
        let p = a.footprint(100.0, Vec2::ZERO);
        let q = a.footprint(100.0, Vec2::ZERO);
        assert_eq!(bytemuck::bytes_of(&p), bytemuck::bytes_of(&q));
        let r = a.footprint(101.0, Vec2::ZERO);
        assert_ne!(p.points[60], r.points[60]);
        for c in 0..3 {
            for j in 1..CURTAIN_POINTS {
                let (u0, u1) = (
                    p.points[c * CURTAIN_POINTS + j - 1][2],
                    p.points[c * CURTAIN_POINTS + j][2],
                );
                assert!((u1 - u0).abs() < 2.0, "curtain {c} point {j}: {u0} {u1}");
            }
        }
        // The first arc's middle lies `distance` out along `direction`
        // (90°: towards -z).
        let mid = p.points[CURTAIN_POINTS / 2];
        assert!(mid[1] < -100.0 && mid[0].abs() < 80.0, "{mid:?}");
    }
}
