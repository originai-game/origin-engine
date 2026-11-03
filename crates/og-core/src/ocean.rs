//! Ocean waves (R5): a wind-driven sea as a sum of [`OCEAN_WAVES`] Gerstner
//! (trochoidal) waves, shared by the renderer, which displaces og.Water's
//! grid with them, and the simulation, which asks the surface height at a
//! point (`og.physics.waterHeight`) for boats and spray. Both evaluate the
//! same waves from the same tick, so what floats matches what is drawn.
//!
//! The waves sample a Pierson–Moskowitz sea for the wind's speed `U`, its
//! peak enhanced as JONSWAP's (γ 3.3): its peak angular frequency `0.877 g / U` (at 10 m/s, waves
//! about 80 m long), its significant height `0.21 U² / g` (2.1 m at
//! 10 m/s) times `swell`, wavelengths from 1.6 times the peak's down to
//! 1/80 of it, one in each of [`OCEAN_WAVES`] bands even in the logarithm
//! of the length and placed at random within it (so no two waves keep a
//! fixed ratio and the sum never settles into a lattice), each the
//! deep-water dispersion `ω = √(g k)`, travelling at a random angle to the
//! wind (a triangular spread, wider the shorter the wave), with fixed
//! phases. The waves under a sixth of the peak's length are then raised to
//! Cox and Munk's measured mean square slope, `0.003 + 0.00512 U` (the
//! spectrum alone leaves the sea glassy). `choppiness` leans each wave's
//! crest by Gerstner's steepness, shared so the sum never folds over.
//!
//! The waves are the world's: they lie over the absolute world's xz (the
//! renderer adds its origin, the simulation og.WorldOrigin's zero), so a
//! sea that follows the camera keeps its waves where they are.
//!
//! Everything is deterministic: the transcendentals are [`crate::math`]'s,
//! and the time and the place enter as each wave's phase,
//! `(φ - ω t + k d · o) mod 2π` at a point `o`, computed in `f64`, so long
//! games and far places keep their precision.

use crate::math::{self, DVec2, Vec2, Vec3};

/// How many waves a sea sums.
pub const OCEAN_WAVES: usize = 32;

/// Standard gravity, m/s².
pub const GRAVITY: f32 = 9.81;

/// Below this wind (m/s) there is no sea.
const CALM: f32 = 0.1;

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
    let mut h = i.wrapping_mul(0x9e37_79b9) ^ c.wrapping_mul(0x85eb_ca6b) ^ 0x0cea_0001;
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb_352d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846c_a68b);
    h ^= h >> 16;
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
        let mut waves = [OceanWave::default(); OCEAN_WAVES];
        let mut weight = [0.0f32; OCEAN_WAVES];
        let mut total = 0.0;
        for (i, w) in waves.iter_mut().enumerate() {
            // Band i of N, even in the logarithm of the length from 1.6 to
            // 1/80 of the peak's, the wave anywhere in its band but its edges.
            let at = (i as f32 + 0.5 + 0.4 * fixed(i as u32, 3)) / OCEAN_WAVES as f32;
            let length = length_p * 1.6 * math::pow(1.0 / 128.0, at);
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
            // A triangular spread about the wind, ±0.6 rad for the longest
            // waves to ±1.4 for the shortest.
            let spread = 0.6 + 0.8 * at;
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
        // The waves under a sixth of the peak's length carry what the eye sees,
        // the slopes: the spectrum's own leave the sea glassy (its mean
        // square slope is about 0.016 at any wind), so they are raised to
        // Cox and Munk's measured slopes, 0.003 + 0.00512 U, times the swell
        // (they add little height).
        let mss = |w: &OceanWave| (w.k * w.amplitude) * (w.k * w.amplitude) * 0.5;
        let short = |w: &OceanWave| w.k > 6.0 * k_p;
        let (long_mss, short_mss) = waves.iter().fold((0.0, 0.0), |(l, s), w| {
            if short(w) {
                (l, s + mss(w))
            } else {
                (l + mss(w), s)
            }
        });
        let want = (0.003 + 0.00512 * speed) * swell.min(1.0);
        if short_mss > 0.0 && want > long_mss + short_mss {
            let lift = ((want - long_mss) / short_mss).sqrt();
            for w in waves.iter_mut().filter(|w| short(w)) {
                w.amplitude *= lift;
            }
        }
        for w in &mut waves {
            // Each wave's steepness Q k a is choppiness / N, at most its own
            // amplitude across: the sum's Jacobian stays above zero.
            let q = (choppiness / (w.k * w.amplitude * OCEAN_WAVES as f32)).min(1.0);
            w.lean = if w.amplitude > 0.0 {
                q * w.amplitude
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
        self.displacement_on_grid(p, phases, 0.0)
    }

    /// [`Ocean::displacement`] as a grid of `cell`-metre cells draws it:
    /// each wave times [`grid_keep`] (0: every wave whole).
    pub fn displacement_on_grid(&self, p: Vec2, phases: &[f32; OCEAN_WAVES], cell: f32) -> Vec3 {
        let mut d = Vec3::ZERO;
        for (w, &phase) in self.waves.iter().zip(phases) {
            let (s, c) = math::sin_cos(w.k * w.direction.dot(p) + phase);
            let keep = grid_keep(w.k, cell);
            d.x += w.direction.x * w.lean * c * keep;
            d.y += w.amplitude * s * keep;
            d.z += w.direction.y * w.lean * c * keep;
        }
        d
    }

    /// The surface's height over its rest plane at the absolute world
    /// point `at` (xz) at `seconds`: the rest point whose displaced point
    /// lies over `at`, found by fixed-point steps (the crests lean, so the
    /// point over `at` came from elsewhere), then its height.
    pub fn height(&self, at: DVec2, seconds: f64) -> f32 {
        self.height_on_grid(at, seconds, 0.0)
    }

    /// [`Ocean::height`] of the surface a grid of `cell`-metre cells draws
    /// ([`Ocean::displacement_on_grid`]): what a body floating on the drawn
    /// sea rides on, its waves too short for the grid left to the normals.
    pub fn height_on_grid(&self, at: DVec2, seconds: f64, cell: f32) -> f32 {
        self.surface_on_grid(at, seconds, cell).height
    }

    /// [`Ocean::height_on_grid`] with the motion of the water at that point
    /// of the surface: its particle's orbital velocity and acceleration
    /// (the time derivatives of its displacement). A floating body is
    /// carried by the one and buoyed by gravity less the other: smooth
    /// where the height at a fixed point is not (a steep crest sweeps past
    /// a fixed point at the wave's phase speed, while the water itself
    /// only turns over in its orbit).
    pub fn surface_on_grid(&self, at: DVec2, seconds: f64, cell: f32) -> SurfacePoint {
        let phases = self.phases(seconds, at);
        let mut rest = Vec2::ZERO;
        for _ in 0..8 {
            let d = self.displacement_on_grid(rest, &phases, cell);
            rest = -Vec2::new(d.x, d.z);
        }
        let mut point = SurfacePoint {
            height: self.displacement_on_grid(rest, &phases, cell).y,
            ..SurfacePoint::default()
        };
        // The phase falls by ω a second: d/dt of (lean cos θ, A sin θ).
        for (w, &phase) in self.waves.iter().zip(&phases) {
            let (s, c) = math::sin_cos(w.k * w.direction.dot(rest) + phase);
            let keep = grid_keep(w.k, cell);
            let (wv, wa) = (keep * w.omega, keep * w.omega * w.omega);
            let lean = Vec3::new(w.direction.x * w.lean, 0.0, w.direction.y * w.lean);
            point.velocity += lean * (s * wv) - Vec3::Y * (w.amplitude * c * wv);
            point.acceleration += -lean * (c * wa) - Vec3::Y * (w.amplitude * s * wa);
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

/// A point of the sea's surface ([`Ocean::surface_on_grid`]): its height
/// over the rest level and its water's velocity and acceleration.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SurfacePoint {
    pub height: f32,
    pub velocity: Vec3,
    pub acceleration: Vec3,
}

/// How much of a wave of wavenumber `k` a grid of `cell`-metre cells
/// displaces: all of one at least eight cells long, none of one under
/// four (the renderer's water.wgsl draws the same fade).
pub fn grid_keep(k: f32, cell: f32) -> f32 {
    let x = ((k * cell - 0.785) / 0.786).clamp(0.0, 1.0);
    1.0 - x * x * (3.0 - 2.0 * x)
}

/// The most cells a water grid has per side (256² cells, 131,072
/// triangles per surface). It caps [`GRID_MAX_CELL`]'s split: a surface
/// longer than 65.5 km gets cells longer than 256 m again.
pub const GRID_MAX_SEGMENTS: u32 = 256;
/// The longest a water grid's cell may be, in world metres, at its even
/// spacing. A rasterizer interpolates a triangle's attributes and depth
/// from its corners in f32, so on a kilometres-long triangle that the near
/// plane clips both land metres off at the far side (og-tests'
/// `terrain-far` lost its shore foam); cells of at most 256 m draw the
/// same thin band everywhere. Shared with the tick so a floating body
/// rides the grid the renderer draws.
pub const GRID_MAX_CELL: f32 = 256.0;

/// Cells per side of a surface `extent` metres across asked for
/// `segments`: more where a cell would pass [`GRID_MAX_CELL`], at most
/// [`GRID_MAX_SEGMENTS`].
pub fn grid_segments(segments: u32, extent: f32) -> u32 {
    let wanted = if extent.is_finite() {
        (extent / GRID_MAX_CELL).ceil() as u32
    } else {
        1
    };
    segments.max(wanted).clamp(1, GRID_MAX_SEGMENTS)
}

/// A sea's grid warp: g in [-1, 1] across the grid to [-1, 1] across the
/// surface, `g (0.15 + 0.85 g²)`, finest at the centre; and its slope,
/// the cell's size over the even grid's.
pub fn grid_warp(g: f32) -> (f32, f32) {
    (g * (0.15 + 0.85 * g * g), 0.15 + 2.55 * g * g)
}

/// The grid cell (m) at `local` (xz from the surface's centre) of a sea
/// `half` across each way, its even cell `even` (extent over segments).
pub fn grid_cell(local: Vec2, half: Vec2, even: f32) -> f32 {
    let slope = |u: f32| {
        let u = u.clamp(-1.0, 1.0);
        let mut g = u;
        for _ in 0..8 {
            let (w, dw) = grid_warp(g);
            g -= (w - u) / dw;
        }
        grid_warp(g).1
    };
    even * slope(local.x / half.x.max(1e-3)).max(slope(local.y / half.y.max(1e-3)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_calm_has_no_sea_and_a_wind_raises_one_of_its_size() {
        assert!(Ocean::new(Vec2::ZERO, 1.0, 1.0).is_none());
        assert!(Ocean::new(Vec2::new(10.0, 0.0), 0.0, 1.0).is_none());
        let sea = Ocean::new(Vec2::new(10.0, 0.0), 1.0, 0.5).expect("a sea");
        // 0.21 U² / g at 10 m/s (2.14 m), and a little more from the
        // short waves raised to Cox and Munk's slopes, which it then has.
        let hs = sea.significant_height();
        assert!((2.14..2.5).contains(&hs), "{hs}");
        let mss: f32 = sea
            .waves
            .iter()
            .map(|w| (w.k * w.amplitude) * (w.k * w.amplitude) * 0.5)
            .sum();
        assert!((mss - (0.003 + 0.0512)).abs() < 1e-4, "{mss}");
        let half = Ocean::new(Vec2::new(0.0, 10.0), 0.5, 0.5).expect("a sea");
        let ratio = half.significant_height() / hs;
        assert!((0.45..0.55).contains(&ratio), "{ratio}");
        // Its longest wave near 1.6 times the peak (83 m at 10 m/s), within
        // its band, travelling down the wind.
        let long = sea.waves[0];
        let length = std::f32::consts::TAU / long.k;
        assert!((113.0..133.0).contains(&length), "{length}");
        assert!(long.direction.x > 0.8);
        // Every wave shorter than the one before, by no fixed ratio.
        let ratios: Vec<f32> = sea.waves.windows(2).map(|w| w[1].k / w[0].k).collect();
        assert!(ratios.iter().all(|&r| r > 1.0));
        let (lo, hi) = ratios
            .iter()
            .fold((f32::MAX, 0.0f32), |(a, b), &r| (a.min(r), b.max(r)));
        assert!(hi - lo > 0.1, "{lo}..{hi}");
        // The peak band carries the most height; the shortest waves little.
        let biggest = (0..OCEAN_WAVES)
            .max_by(|&a, &b| sea.waves[a].amplitude.total_cmp(&sea.waves[b].amplitude))
            .unwrap();
        assert!((1..=7).contains(&biggest), "the peak is wave {biggest}");
        assert!(sea.waves[OCEAN_WAVES - 1].amplitude < 0.1 * sea.waves[biggest].amplitude);
    }

    #[test]
    fn the_surface_never_folds_and_the_height_is_what_is_drawn_over_the_point() {
        let sea = Ocean::new(Vec2::new(6.0, 8.0), 1.0, 1.0).expect("a sea");
        let steep: f32 = sea.waves.iter().map(|w| w.k * w.lean).sum();
        assert!(steep <= 1.0 + 1e-5, "Σ Q k a = {steep}");
        let seconds = 1234.5;
        // Far from the world's zero, as og.WorldOrigin lets a world go.
        let zero = DVec2::new(250_000.0, -80_000.0);
        let phases = sea.phases(seconds, zero);
        for i in 0..40 {
            let p = Vec2::new(i as f32 * 7.3 - 140.0, i as f32 * -3.1 + 20.0);
            let h = sea.height(zero + p.as_dvec2(), seconds);
            // The rest point the height came from is displaced over p.
            let mut rest = p;
            for _ in 0..8 {
                let d = sea.displacement(rest, &phases);
                rest = p - Vec2::new(d.x, d.z);
            }
            let d = sea.displacement(rest, &phases);
            assert!((rest + Vec2::new(d.x, d.z) - p).length() < 0.05, "at {p}");
            assert!((d.y - h).abs() < 2e-3, "{} vs {h}", d.y);
            assert!(h.abs() < 3.0 * sea.significant_height());
        }
    }

    #[test]
    fn the_grid_drops_only_its_too_short_waves() {
        let sea = Ocean::new(Vec2::new(-3.0, -8.5), 1.0, 0.6).expect("a sea");
        let at = DVec2::new(12.0, -40.0);
        assert_eq!(sea.height_on_grid(at, 3.0, 0.0), sea.height(at, 3.0));
        // A 3.5 m cell keeps the swell, drops the ripples: close to the
        // whole sea, but not it; a 1 km cell keeps nothing.
        let fine = sea.height_on_grid(at, 3.0, 3.5);
        assert!(fine != sea.height(at, 3.0) && (fine - sea.height(at, 3.0)).abs() < 0.6);
        assert_eq!(sea.height_on_grid(at, 3.0, 1000.0), 0.0);
        // The warp: 0.15 of the even cell at the centre, 2.7 at the edge.
        assert!((grid_cell(Vec2::ZERO, Vec2::splat(3000.0), 10.0) - 1.5).abs() < 1e-4);
        assert!(
            (grid_cell(Vec2::new(0.0, -3000.0), Vec2::splat(3000.0), 10.0) - 27.0).abs() < 1e-3
        );
        let (w, _) = grid_warp(0.4);
        let c = grid_cell(Vec2::new(w * 3000.0, 0.0), Vec2::splat(3000.0), 10.0);
        assert!((c - grid_warp(0.4).1 * 10.0).abs() < 1e-3);
        assert_eq!(grid_segments(256, 6000.0), 256);
        assert_eq!(grid_segments(16, 24000.0), 94);
    }

    #[test]
    fn the_surface_velocity_is_its_particles_motion() {
        // The water at the point moves as its rest point's displacement
        // does over a short time (a central difference), not as the height
        // at the fixed point changes.
        let sea = Ocean::new(Vec2::new(7.0, -18.7), 1.0, 1.0).expect("a sea");
        let at = DVec2::new(-12.0, -30.0);
        let (t, h, cell) = (5.0f64, 1e-3f64, 3.6f32);
        let phases = sea.phases(t, at);
        let mut rest = Vec2::ZERO;
        for _ in 0..8 {
            let d = sea.displacement_on_grid(rest, &phases, cell);
            rest = -Vec2::new(d.x, d.z);
        }
        let moved = |dt: f64| sea.displacement_on_grid(rest, &sea.phases(t + dt, at), cell);
        let fd = (moved(h) - moved(-h)) / (2.0 * h) as f32;
        // (A second difference wants a longer step in f32.)
        let h2 = 0.02f64;
        let fa = (moved(h2) - moved(0.0) * 2.0 + moved(-h2)) / (h2 * h2) as f32;
        let p = sea.surface_on_grid(at, t, cell);
        assert_eq!(p.height, sea.height_on_grid(at, t, cell));
        let v = p.velocity;
        assert!(
            (v - fd).length() < 0.02 * fd.length().max(1.0),
            "{v} vs {fd}"
        );
        let a = p.acceleration;
        assert!(
            (a - fa).length() < 0.05 * fa.length().max(1.0),
            "{a} vs {fa}"
        );
    }

    #[test]
    fn phases_wrap_without_losing_time() {
        let sea = Ocean::new(Vec2::new(10.0, 0.0), 1.0, 0.0).expect("a sea");
        // A day of ticks at 60 Hz and the same moment a cycle later agree
        // to the f32 phase's own precision.
        let t = 86_400.0;
        let w = sea.waves[3];
        let period = std::f64::consts::TAU / f64::from(w.omega);
        let a = sea.phases(t, DVec2::ZERO)[3];
        let b = sea.phases(t + period * 1000.0, DVec2::ZERO)[3];
        let gap = (a - b).abs();
        assert!(gap.min(std::f32::consts::TAU - gap) < 1e-3, "{a} vs {b}");
        // The surface moves with time: a crest travels down the wind.
        let p = DVec2::new(3.0, 0.0);
        assert_ne!(sea.height(p, 0.0), sea.height(p, 0.5));
    }
}
