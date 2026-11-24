//! Ocean waves (R5): a wind-driven sea as one function of the place and the
//! time, a sum of [`OCEAN_WAVES`] Gerstner (trochoidal) waves. The
//! simulation asks it the surface's height and motion at a point
//! (`og.physics.waterHeight`, og.Buoyancy) and the renderer draws the same
//! function at the same tick, so what floats is what is drawn. The
//! function knows no grid, no surface size and no camera: every wave is
//! whole everywhere. What a renderer cannot resolve somewhere it leaves out
//! by its own rule; what it adds, the ripples shorter than the shortest
//! wave here, only shades the surface and never moves it.
//!
//! The waves sample a Pierson–Moskowitz sea for the wind's speed `U`, its
//! peak enhanced as JONSWAP's (γ 3.3): its peak angular frequency `0.877 g / U` (at 10 m/s, waves
//! about 80 m long), its significant height `0.21 U² / g` (2.1 m at
//! 10 m/s) times `swell`, wavelengths from 1.6 times the peak's down to
//! 1/32 of it or 1 m, whichever is shorter ([`shortest_wavenumber`]), one in
//! each of [`OCEAN_WAVES`] bands even in the logarithm of the length and
//! placed at random within it (so no two waves keep a fixed ratio and the
//! sum never settles into a lattice), each the deep-water dispersion
//! `ω = √(g k)`, travelling at a random angle to the wind (a triangular
//! spread, wider the shorter the wave), with fixed phases. These carry all
//! of the sea's height a surface drawn in vertices can show: the ripples
//! shorter than a metre stand millimetres to a couple of centimetres. Those
//! carry most of its slope and are too many for a sum: they are the
//! renderer's shading, drawn from the same spectrum past the shortest wave
//! here. `choppiness` leans each wave's crest by Gerstner's steepness,
//! shared by the waves' heights so the sum never folds over.
//!
//! The waves are the world's: they lie over the absolute world's xz (the
//! renderer adds its origin, the simulation og.WorldOrigin's zero), so a
//! sea that follows the camera keeps its waves where they are.
//!
//! Everything is deterministic: the transcendentals are [`og_core::math`]'s,
//! and the time and the place enter as each wave's phase,
//! `(φ - ω t + k d · o) mod 2π` at a point `o`, computed in `f64`, so long
//! games and far places keep their precision.

pub use crate::shore::{Shore, ShorePoint, ShoreTrains};
use og_core::math::{self, DVec2, Vec2, Vec3};

/// How many waves a sea sums: eight to twelve to each factor of e in
/// length over its span (from 1.6 times the peak's length down to a 32nd
/// of it in a breeze, to a metre, a 330th of it, at 20 m/s).
pub const OCEAN_WAVES: usize = 48;

/// Standard gravity, m/s².
pub const GRAVITY: f32 = 9.81;

/// Below this wind (m/s) there is no sea.
const CALM: f32 = 0.1;

/// The peak's length over the shortest wave's, at most. The waves shorter
/// than a 32nd of the peak hold about a hundredth of the sea's height
/// (1.1% of its significant height in RMS at 10 m/s, 1.6% at 20).
const SHORTEST: f32 = 32.0;

/// The shortest wave's length (m), at most: in a wind whose 32nd of the
/// peak is longer (over 6.2 m/s) the waves reach down to here, so every
/// wave a surface's vertices can draw is one the tick floats bodies on
/// (a clipmap's finest cells, an eighth of a metre, draw a metre-long wave
/// whole). Shorter waves stand a centimetre or so even in a gale.
const SHORTEST_LENGTH: f32 = 1.0;

/// One Gerstner wave of a sea.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OceanWave {
    /// The unit direction it travels, in the surface's xz.
    pub direction: Vec2,
    /// Its wavenumber, 2π over its length (rad/m).
    pub k: f32,
    /// Its angular frequency, √(g k) (rad/s).
    pub omega: f32,
    /// Its height amplitude (m).
    pub amplitude: f32,
    /// How far its crests lean along `direction` (m): Gerstner's Q · a.
    pub lean: f32,
    /// Its phase at time zero (rad).
    pub phase: f32,
}

/// A sea: its waves. See the module documentation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ocean {
    pub waves: [OceanWave; OCEAN_WAVES],
}

/// A fixed value in [-1, 1] for wave `i` and channel `c` (an integer
/// hash: the same on every host).
fn fixed(i: u32, c: u32) -> f32 {
    let h = og_core::hash::lowbias32(
        i.wrapping_mul(0x9e37_79b9) ^ c.wrapping_mul(0x85eb_ca6b) ^ 0x0cea_0001,
    );
    (h >> 8) as f32 / (1u32 << 23) as f32 - 1.0
}

impl Ocean {
    /// The sea the `wind` (its direction in the surface's xz and its speed
    /// in m/s as its length) raises, its height times `swell` and its crests
    /// leaning by `choppiness` (0: round sine swell, 1: the most that never
    /// folds). `None` in a calm (wind under 0.1 m/s) or without swell.
    pub fn new(wind: Vec2, swell: f32, choppiness: f32) -> Option<Ocean> {
        let speed = wind.length();
        if !speed.is_finite() || !swell.is_finite() || speed < CALM || swell <= 0.0 {
            return None;
        }
        let choppiness = if choppiness.is_finite() {
            choppiness.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let heading = math::atan2(wind.y, wind.x);
        let omega_p = 0.877 * GRAVITY / speed;
        let k_p = omega_p * omega_p / GRAVITY;
        let length_p = std::f32::consts::TAU / k_p;
        let span = 1.6 * length_p * shortest_wavenumber(speed) / std::f32::consts::TAU;
        let mut waves = [OceanWave::default(); OCEAN_WAVES];
        let mut weight = [0.0f32; OCEAN_WAVES];
        let mut total = 0.0;
        for (i, w) in waves.iter_mut().enumerate() {
            // Band i of N, even in the logarithm of the length from 1.6 times
            // the peak's to the shortest, the wave anywhere in its band but its edges.
            let at = (i as f32 + 0.5 + 0.4 * fixed(i as u32, 3)) / OCEAN_WAVES as f32;
            let length = length_p * 1.6 * math::pow(1.0 / span, at);
            let k = std::f32::consts::TAU / length;
            let omega = (GRAVITY * k).sqrt();
            // Pierson–Moskowitz, S(ω) ∝ ω⁻⁵ exp(-1.25 (ω_p/ω)⁴), with
            // JONSWAP's peak enhancement γ^exp(-(ω - ω_p)² / 2σ²ω_p²) (γ 3.3,
            // σ 0.07 below the peak, 0.09 above: the height gathered into
            // swells that read), over bands whose width grows with ω (the
            // lengths are spaced evenly in their logarithm), so each wave's
            // share is S(ω) ω.
            let r = omega_p / omega;
            let sw = if omega <= omega_p { 0.07 } else { 0.09 };
            let off = (omega - omega_p) / (sw * omega_p);
            let peak = math::exp(math::exp(-0.5 * off * off) * math::ln(3.3));
            let s = math::pow(omega, -4.0) * math::exp(-1.25 * r * r * r * r) * peak;
            weight[i] = s;
            total += s;
            // A triangular spread about the wind, ±0.9 rad for the longest
            // waves to ±2 for the shortest (a standard deviation of 21° to
            // 47°, as the directional spectra measured at sea widen from
            // the peak: Hasselmann et al. 1980).
            let spread = 0.9 + 1.1 * at;
            let angle = heading + spread * 0.5 * (fixed(i as u32, 0) + fixed(i as u32, 2));
            let (sin, cos) = math::sin_cos(angle);
            *w = OceanWave {
                direction: Vec2::new(cos, sin),
                k,
                omega,
                amplitude: 0.0,
                lean: 0.0,
                phase: std::f32::consts::PI * fixed(i as u32, 1),
            };
        }
        // The significant height: 4 × the surface's standard deviation.
        let sigma = 0.21 * speed * speed / GRAVITY * swell / 4.0;
        for (w, s) in waves.iter_mut().zip(weight) {
            w.amplitude = sigma * (2.0 * s / total).sqrt();
        }
        // The crests lean on the waves down to a 32nd of the peak's length,
        // their steepnesses Q k a summing to choppiness so the sum's Jacobian
        // stays above zero: each leans its amplitude times min(1, λ / k) (Q at
        // most 1, its own amplitude across), λ the wavenumber at which the
        // sum comes to choppiness, so the long waves that make the crests
        // lean whole. The shorter waves a gale's span adds (to a metre) are
        // round: they carry height a body rides, not lean; leaning, they
        // bunched the surface metre by metre and its whitecaps broke up into
        // a speckle. The waves are in order of wavenumber, so λ lies between
        // two of them.
        let leaning = SHORTEST * k_p * 1.0001;
        let mut whole = 0.0; // Σ k a of the waves at Q 1.
        let mut rest: f32 = waves
            .iter()
            .filter(|w| w.k <= leaning)
            .map(|w| w.amplitude)
            .sum();
        let mut lambda = f32::INFINITY;
        for w in waves.iter().filter(|w| w.k <= leaning) {
            let l = (choppiness - whole) / rest;
            if l <= w.k {
                lambda = l;
                break;
            }
            whole += w.k * w.amplitude;
            rest -= w.amplitude;
        }
        for w in &mut waves {
            w.lean = if w.k <= leaning {
                w.amplitude * (lambda / w.k).min(1.0)
            } else {
                0.0
            };
        }
        Some(Ocean { waves })
    }

    /// Each wave's phase at `seconds` and the absolute world point `at`,
    /// `(φ - ω t + k d · at) mod 2π`, in `f64`: [`Ocean::displacement`]
    /// then takes points relative to `at` (the renderer's uniform holds
    /// these at its origin).
    pub fn phases(&self, seconds: f64, at: DVec2) -> [f32; OCEAN_WAVES] {
        let tau = std::f64::consts::TAU;
        self.waves.map(|w| {
            let along = w.direction.as_dvec2().dot(at) * f64::from(w.k);
            (f64::from(w.phase) - f64::from(w.omega) * seconds + along).rem_euclid(tau) as f32
        })
    }

    /// The surface's displacement (x, height, z) at rest point `p`, relative
    /// to the point `phases` were taken at ([`Ocean::phases`]).
    pub fn displacement(&self, p: Vec2, phases: &[f32; OCEAN_WAVES]) -> Vec3 {
        self.displaced(p, phases, &Shoaling::OPEN)
    }

    /// The displacement at rest point `p` with the seabed's `shoal` there.
    fn displaced(&self, p: Vec2, phases: &[f32; OCEAN_WAVES], shoal: &Shoaling) -> Vec3 {
        let mut d = Vec3::ZERO;
        for ((w, &phase), &f) in self.waves.iter().zip(phases).zip(&shoal.open) {
            let (s, c) = math::sin_cos(w.k * w.direction.dot(p) + phase);
            d.x += w.direction.x * w.lean * c * f;
            d.y += w.amplitude * s * f;
            d.z += w.direction.y * w.lean * c * f;
        }
        if shoal.amplitude > 0.0 {
            let (s, c) = math::sin_cos(shoal.phase);
            d.x += shoal.inshore.x * shoal.lean * c;
            d.y += shoal.amplitude * s;
            d.z += shoal.inshore.y * shoal.lean * c;
        }
        d
    }

    /// How the seabed `at` a point (`None`: the open sea) changes this sea
    /// there, with `trains` rolling in: each wave shoaled by `tanh(k d)`
    /// (Crest's and the TMA spectrum's depth attenuation: none in water
    /// deeper than half its length, all of it on the beach), the trains
    /// risen in the shallows, and the whole capped at the breaking height,
    /// a significant height of 0.78 d (McCowan's limit, which the surf zone
    /// saturates at). Near the seabed grid's edge it fades to the open sea.
    /// `trains_phase` is their (φ - ω t) mod 2π now ([`Ocean::train_phase`]).
    pub fn shoaling(
        &self,
        at: Option<ShorePoint>,
        trains: &ShoreTrains,
        trains_phase: f32,
    ) -> Shoaling {
        let Some(sp) = at else {
            return Shoaling::OPEN;
        };
        let depth = sp.depth.max(0.0);
        let mut out = Shoaling::OPEN;
        let mut var = 0.0;
        for (f, w) in out.open.iter_mut().zip(&self.waves) {
            *f = 1.0 + (tanh(w.k * depth) - 1.0) * sp.inside;
            var += (w.amplitude * *f) * (w.amplitude * *f) * 0.5;
        }
        let a_s = trains.amplitude * trains.rise(depth) * sp.coherence * sp.inside;
        var += a_s * a_s * 0.5;
        let hs = 4.0 * var.sqrt();
        let limit = 0.78 * depth;
        let cap = if hs > limit {
            1.0 + (limit / hs - 1.0) * sp.inside
        } else {
            1.0
        };
        let mut open_steep = 0.0;
        for (f, w) in out.open.iter_mut().zip(&self.waves) {
            *f *= cap;
            open_steep += w.k * w.lean * *f;
        }
        out.amplitude = a_s * cap;
        if out.amplitude > 0.0 {
            // Gerstner's steepness Q = k · lean, what the open sea leaves
            // of 0.95 so the sum never folds over.
            let q = trains.steepness(depth).min((0.95 - open_steep).max(0.0));
            out.lean = (q / trains.k).min(3.0 * out.amplitude);
            out.k = trains.k;
            out.omega = trains.omega;
            out.inshore = sp.inshore;
            // Their crests are the lines of equal distance to the waterline,
            // travelling inshore (down the distance).
            out.phase = trains_phase - trains.k * sp.distance;
        }
        out
    }

    /// The shore trains' phase (φ - ω t) mod 2π at `seconds`, in f64.
    pub fn train_phase(trains: &ShoreTrains, seconds: f64) -> f32 {
        (f64::from(trains.phase) - f64::from(trains.omega) * seconds)
            .rem_euclid(std::f64::consts::TAU) as f32
    }

    /// The surface's height over its rest plane at the absolute world
    /// point `at` (xz) at `seconds`: the rest point whose displaced point
    /// lies over `at`, found by fixed-point steps (the crests lean, so the
    /// point over `at` came from elsewhere), then its height.
    pub fn height(&self, at: DVec2, seconds: f64) -> f32 {
        self.surface(at, seconds).height
    }

    /// [`Ocean::height`] with the motion of the water at that point of the
    /// surface: its particle's orbital velocity and acceleration (the time
    /// derivatives of its displacement). A floating body is carried by the
    /// one and buoyed by gravity less the other: smooth where the height at
    /// a fixed point is not (a steep crest sweeps past a fixed point at the
    /// wave's phase speed, while the water itself only turns over in its
    /// orbit).
    pub fn surface(&self, at: DVec2, seconds: f64) -> SurfacePoint {
        self.surface_near(at, seconds, None)
    }

    /// [`Ocean::surface`] near a `coast` (`None`: the open sea): every wave
    /// shoaled by the seabed under the rest point it comes from
    /// ([`Ocean::shoaling`]), the shore trains rolling in on top.
    pub fn surface_near(&self, at: DVec2, seconds: f64, coast: Option<&Coast<'_>>) -> SurfacePoint {
        let phases = self.phases(seconds, at);
        let trains_phase = coast.map_or(0.0, |c| Ocean::train_phase(&c.trains, seconds));
        let frame = coast.map(|c| (at - c.zero).as_vec2());
        let shoal_at = |rest: Vec2| match (coast, frame) {
            (Some(c), Some(f)) => self.shoaling(c.shore.sample(f + rest), &c.trains, trains_phase),
            _ => Shoaling::OPEN,
        };
        let mut rest = Vec2::ZERO;
        for _ in 0..8 {
            let d = self.displaced(rest, &phases, &shoal_at(rest));
            rest = -Vec2::new(d.x, d.z);
        }
        let shoal = shoal_at(rest);
        let mut point = SurfacePoint {
            height: self.displaced(rest, &phases, &shoal).y,
            ..SurfacePoint::default()
        };
        // The phase falls by ω a second: d/dt of (lean cos θ, A sin θ).
        let mut orbit = |dir: Vec2, omega: f32, amplitude: f32, lean: f32, theta: f32, f: f32| {
            let (s, c) = math::sin_cos(theta);
            let (wv, wa) = (f * omega, f * omega * omega);
            let lean = Vec3::new(dir.x * lean, 0.0, dir.y * lean);
            point.velocity += lean * (s * wv) - Vec3::Y * (amplitude * c * wv);
            point.acceleration += -lean * (c * wa) - Vec3::Y * (amplitude * s * wa);
        };
        for ((w, &phase), &f) in self.waves.iter().zip(&phases).zip(&shoal.open) {
            let theta = w.k * w.direction.dot(rest) + phase;
            orbit(w.direction, w.omega, w.amplitude, w.lean, theta, f);
        }
        if shoal.amplitude > 0.0 {
            orbit(
                shoal.inshore,
                shoal.omega,
                shoal.amplitude,
                shoal.lean,
                shoal.phase,
                1.0,
            );
        }
        point
    }

    /// The significant wave height the sea was made for (m).
    pub fn significant_height(&self) -> f32 {
        4.0 * (self
            .waves
            .iter()
            .map(|w| w.amplitude * w.amplitude * 0.5)
            .sum::<f32>())
        .sqrt()
    }
}

/// A point of the sea's surface ([`Ocean::surface`]): its height
/// over the rest level and its water's velocity and acceleration.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SurfacePoint {
    pub height: f32,
    pub velocity: Vec3,
    pub acceleration: Vec3,
}

/// A sea near a shore ([`Ocean::surface_near`]): the seabed under it, the
/// trains rolling in on it, and the absolute world xz of the zero of the
/// frame the seabed's grid lies in (og.WorldOrigin's).
#[derive(Clone, Copy, Debug)]
pub struct Coast<'a> {
    pub shore: &'a Shore,
    pub trains: ShoreTrains,
    pub zero: DVec2,
}

/// What the seabed under a point does to the sea there ([`Ocean::shoaling`]):
/// each open wave's factor (amplitude and lean), and the shore train's
/// amplitude, lean, wavenumber, frequency, direction and phase there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shoaling {
    pub open: [f32; OCEAN_WAVES],
    pub amplitude: f32,
    pub lean: f32,
    pub k: f32,
    pub omega: f32,
    pub inshore: Vec2,
    pub phase: f32,
}

impl Shoaling {
    /// The open sea: every wave whole, no trains.
    pub const OPEN: Shoaling = Shoaling {
        open: [1.0; OCEAN_WAVES],
        amplitude: 0.0,
        lean: 0.0,
        k: 0.0,
        omega: 0.0,
        inshore: Vec2::ZERO,
        phase: 0.0,
    };
}

/// tanh, deterministic on every host ([`math::exp`]'s).
pub fn tanh(x: f32) -> f32 {
    if x >= 0.0 {
        1.0 - 2.0 / (math::exp(2.0 * x) + 1.0)
    } else {
        -tanh(-x)
    }
}

/// The wavenumber (rad/m) past which a sea at wind `speed` (m/s) has no
/// waves: the short end of its last band, 1/32 of the peak's length or a
/// metre, whichever is shorter (0.65 m at 5 m/s, 1 m from 6.2 m/s). What a
/// renderer adds on top starts here, so no wave is drawn twice.
pub fn shortest_wavenumber(speed: f32) -> f32 {
    let omega_p = 0.877 * GRAVITY / speed.max(CALM);
    (SHORTEST * omega_p * omega_p / GRAVITY).max(std::f32::consts::TAU / SHORTEST_LENGTH)
}

/// The mean square slope of a sea at wind `speed` (m/s) Cox and Munk (1954)
/// measured from the sun's glitter: the whole of a clean sea's.
pub fn clean_slopes(speed: f32) -> f32 {
    0.003 + 0.00512 * speed
}
