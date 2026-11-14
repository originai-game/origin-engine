//! The physical sky (the R4 round): single scattering by air (Rayleigh)
//! and aerosols (Mie) with ozone absorption in a spherical atmosphere,
//! plus Hillaire's multiple-scattering term ("A Scalable and Production
//! Ready Sky and Atmosphere Rendering Technique", 2020), marched per pixel
//! by `shaders/atmosphere.wgsl` and, with the same formulas, here on the
//! CPU: the sun's colour through the air ([`PhysicalAtmosphere::sun_transmittance`])
//! for a day-night cycle's light, the multiple-scattering table the shader
//! reads, and the reference the tests check the shader's numbers against.
//!
//! Lengths are kilometres inside the model; coefficients are given per
//! megametre (Hillaire's per-metre values times 10^6). The optical depth
//! towards the sun is Schüler's approximation of the Chapman function
//! (GPU Pro 3, "An Approximation to the Chapman Grazing-Incidence
//! Function"), so neither side needs a lookup texture: the shader keeps to
//! the frame uniform and the sky pass's existing bindings.

use glam::{Vec2, Vec3};
use std::f32::consts::PI;

/// The multiple-scattering table's entries per height (over the sun's
/// zenith cosine, -1 to 1).
pub(crate) const MS_ENTRIES: usize = 16;
/// The two heights (km) the table is taken at; the shader blends between
/// them and holds the second above it.
pub(crate) const MS_HEIGHTS: [f32; 2] = [0.0, 8.0];
/// The vec4s the GPU block holds: six of parameters, then the table.
pub(crate) const SCATTER_VEC4S: usize = 6 + 2 * MS_ENTRIES;
/// Radiance per unit of the sun's illuminance times this is the engine's
/// sky brightness: the sun's illuminance above the air in the day-night
/// light's units (3 at noon through the air, about 3.5 above it), with a
/// camera's allowance for a sky that reads blue (about twice that): a
/// clear noon zenith about half as bright as sunlit grey ground and a
/// horizon a few times the zenith, and the sky's light (the image-based
/// ambient) under a third of the sun's, as outdoors. (20 drew the sky and
/// its ambient two and a half times too bright: a washed-out noon.)
const SKY_SCALE: f32 = 8.0;
/// The ozone layer: a tent from 10 to 40 km peaking at 25 km (15 km of
/// column at peak density).
const OZONE_CENTER: f32 = 25.0;
const OZONE_HALF_WIDTH: f32 = 15.0;

/// A planet's atmosphere for the physical sky; the default is Earth's
/// (Hillaire 2020). See the module documentation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhysicalAtmosphere {
    /// Air's scattering per Mm at the ground, per channel (Earth: 5.802,
    /// 13.558, 33.1, which makes the sky blue and sunsets red).
    pub rayleigh: Vec3,
    /// Air's scale height in km (its density falls by e over it).
    pub rayleigh_height: f32,
    /// Aerosols' scattering per Mm at the ground (3.996: a clear day;
    /// 10 to 40: haze and smog, a whiter sky and a wider glow around the sun).
    pub mie: f32,
    /// Aerosols' absorption per Mm at the ground.
    pub mie_absorption: f32,
    /// Aerosols' scale height in km.
    pub mie_height: f32,
    /// Which way aerosols scatter (Cornette–Shanks g): towards 1 the glow
    /// around the sun tightens.
    pub mie_anisotropy: f32,
    /// Ozone's absorption per Mm at its peak, 25 km up (Earth: 0.650,
    /// 1.881, 0.085, which keeps the twilight zenith blue).
    pub ozone: Vec3,
    /// The ground's albedo, lighting the air from below.
    pub ground_albedo: Vec3,
    /// The planet's radius and the atmosphere's thickness, in km.
    pub planet_radius: f32,
    pub thickness: f32,
    /// The viewer's height above the ground at the world's y = 0, in metres
    /// (the camera's height is added).
    pub altitude: f32,
    /// The sky's radiance with the sun far below the horizon (airglow and
    /// starlight), so a night is dark blue rather than black.
    pub night: Vec3,
}

impl Default for PhysicalAtmosphere {
    fn default() -> Self {
        PhysicalAtmosphere::EARTH
    }
}

impl PhysicalAtmosphere {
    /// Earth's atmosphere (Hillaire 2020's coefficients).
    pub const EARTH: PhysicalAtmosphere = PhysicalAtmosphere {
        rayleigh: Vec3::new(5.802, 13.558, 33.1),
        rayleigh_height: 8.0,
        mie: 3.996,
        mie_absorption: 0.444,
        mie_height: 1.2,
        mie_anisotropy: 0.8,
        ozone: Vec3::new(0.650, 1.881, 0.085),
        ground_albedo: Vec3::splat(0.3),
        planet_radius: 6360.0,
        thickness: 100.0,
        altitude: 0.0,
        // Above airglow and starlight's radiance (a few times it): the
        // eye's night vision, so the land under a twilight or night sky
        // keeps a deep blue-grey ambient, a few percent of noon's after
        // the eye opens up, rather than clipping to black.
        night: Vec3::new(0.0022, 0.0032, 0.0075),
    };

    /// The model in km units, with every input made safe.
    fn model(&self) -> Model {
        let r = self.planet_radius.clamp(10.0, 1.0e5);
        Model {
            rayleigh: self.rayleigh.max(Vec3::ZERO) * 1e-3,
            mie: self.mie.max(0.0) * 1e-3,
            mie_ext: (self.mie.max(0.0) + self.mie_absorption.max(0.0)) * 1e-3,
            ozone: self.ozone.max(Vec3::ZERO) * 1e-3,
            hr: self.rayleigh_height.clamp(0.05, 100.0),
            hm: self.mie_height.clamp(0.05, 100.0),
            g: self.mie_anisotropy.clamp(-0.95, 0.95),
            bottom: r,
            top: r + self.thickness.clamp(1.0, 1000.0),
            albedo: self.ground_albedo.clamp(Vec3::ZERO, Vec3::ONE),
        }
    }

    /// How much of the sunlight reaches a viewer `altitude` metres up with
    /// the sun at `sun_y` (the sine of its elevation), per channel: the
    /// colour of the sun's light (0 once the planet hides it). The day-night
    /// cycle multiplies its light by this.
    pub fn sun_transmittance(&self, altitude: f32, sun_y: f32) -> Vec3 {
        let m = self.model();
        m.sun_transmittance(
            m.bottom + (altitude * 1e-3).max(0.001),
            sun_y.clamp(-1.0, 1.0),
        )
    }

    /// The sky's radiance along unit `dir` from a viewer `altitude` metres
    /// up, the sun along unit `sun` with illuminance 1, in the engine's
    /// units (before `SkyLook::brightness`): single and multiple scattering
    /// and the lit ground, without the sun's disc or the night floor. The
    /// CPU twin of `atmosphere.wgsl`'s `physical_sky`, at `steps` samples.
    pub fn radiance(&self, altitude: f32, dir: Vec3, sun: Vec3, steps: u32) -> Vec3 {
        let m = self.model();
        let ms = self.ms_cached();
        let h0 = (altitude * 1e-3).max(0.001);
        m.march(
            Vec3::new(0.0, m.bottom + h0, 0.0),
            dir.normalize_or(Vec3::Y),
            sun.normalize_or(Vec3::Y),
            steps.max(1),
            Some(&ms),
        )
        .0 * SKY_SCALE
    }

    /// The block `atmosphere.wgsl` reads ([`SCATTER_VEC4S`] vec4s): the
    /// coefficients, the radii, the scale, the ground, the night, then the
    /// multiple-scattering table at [`MS_HEIGHTS`]. Row 0's w is 1 (on).
    pub(crate) fn gpu(&self) -> [[f32; 4]; SCATTER_VEC4S] {
        let m = self.model();
        let mut out = [[0.0; 4]; SCATTER_VEC4S];
        out[0] = m.rayleigh.extend(1.0).to_array();
        out[1] = [m.mie, m.mie_ext, 1.0 / m.hr, 1.0 / m.hm];
        out[2] = m.ozone.extend(m.g).to_array();
        out[3] = [m.bottom, m.top, self.altitude.max(0.0) * 1e-3, SKY_SCALE];
        out[4] = m.albedo.extend(0.0).to_array();
        out[5] = self.night.max(Vec3::ZERO).extend(0.0).to_array();
        for (i, v) in self.ms_cached().0.iter().enumerate() {
            out[6 + i] = v.extend(0.0).to_array();
        }
        out
    }
}

impl PhysicalAtmosphere {
    /// The multiple-scattering table, integrated once per atmosphere (a
    /// few milliseconds) and kept while the atmosphere stays the same.
    fn ms_cached(&self) -> MsTable {
        static CACHE: std::sync::Mutex<Option<(PhysicalAtmosphere, MsTable)>> =
            std::sync::Mutex::new(None);
        let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((a, t)) = cache.as_ref()
            && a == self
        {
            return t.clone();
        }
        let t = self.model().ms_table();
        *cache = Some((*self, t.clone()));
        t
    }
}

/// The atmosphere in km.
struct Model {
    rayleigh: Vec3,
    mie: f32,
    mie_ext: f32,
    ozone: Vec3,
    hr: f32,
    hm: f32,
    g: f32,
    bottom: f32,
    top: f32,
    albedo: Vec3,
}

/// Where a ray from `o` along unit `d` meets the sphere of radius `r`
/// about the origin: (near, far), or `None` when it misses.
fn sphere(o: Vec3, d: Vec3, r: f32) -> Option<Vec2> {
    let b = o.dot(d);
    let c = o.length_squared() - r * r;
    let disc = b * b - c;
    if disc < 0.0 {
        return None;
    }
    let s = disc.sqrt();
    Some(Vec2::new(-b - s, -b + s))
}

/// The Chapman column: the optical depth (in units of the scale height
/// times the ground density) of an exponential layer of scale height `h`
/// along a ray from radius `r` with zenith cosine `mu` to infinity, over a
/// planet of radius `bottom`. Above the horizon the asymptotic form
/// sqrt(πx/2)·exp(y²)·erfc(y), y = sqrt(x/2)·mu, with Ren and MacKenzie's
/// approximation of exp(y²)·erfc(y); below it Schüler's identity (twice
/// the horizontal column from the tangent point, less the column the other
/// way).
fn chapman(r: f32, mu: f32, h: f32, bottom: f32) -> f32 {
    let up = |x: f32, mu: f32| {
        let y = (0.5 * x).sqrt() * mu;
        let s = PI.sqrt();
        (0.5 * PI * x).sqrt() * 2.911 / (1.911 * s * y + (PI * y * y + 8.474).sqrt())
    };
    let x = r / h;
    let a = ((bottom - r) / h).min(0.0).exp();
    if mu >= 0.0 {
        up(x, mu) * a
    } else {
        let r0 = r * (1.0 - mu * mu).max(0.0).sqrt();
        2.0 * up(r0 / h, 0.0) * ((bottom - r0) / h).min(0.0).exp() - up(x, -mu) * a
    }
}

/// The ozone column (in km of peak density) along a ray from radius `r`
/// with zenith cosine `mu`: the share of the tent above the ray's start,
/// slanted as through a thin shell at the peak (or at the start, above it).
fn ozone_column(r: f32, mu: f32, bottom: f32) -> f32 {
    let h = r - bottom;
    let lo = OZONE_CENTER - OZONE_HALF_WIDTH;
    let hi = OZONE_CENTER + OZONE_HALF_WIDTH;
    // The tent's column above h.
    let above = if h <= lo {
        OZONE_HALF_WIDTH
    } else if h >= hi {
        0.0
    } else if h < OZONE_CENTER {
        let u = (h - lo) / OZONE_HALF_WIDTH;
        OZONE_HALF_WIDTH * (1.0 - 0.5 * u * u)
    } else {
        let u = (hi - h) / OZONE_HALF_WIDTH;
        OZONE_HALF_WIDTH * 0.5 * u * u
    };
    let rs = r.max(bottom + OZONE_CENTER);
    let s = r * r * (1.0 - mu * mu);
    let slant = rs / (rs * rs - s).max(rs * rs * 6e-4).sqrt();
    above * slant.min(40.0)
}

fn rayleigh_phase(nu: f32) -> f32 {
    3.0 / (16.0 * PI) * (1.0 + nu * nu)
}

fn mie_phase(nu: f32, g: f32) -> f32 {
    let g2 = g * g;
    3.0 / (8.0 * PI) * (1.0 - g2) * (1.0 + nu * nu)
        / ((2.0 + g2) * (1.0 + g2 - 2.0 * g * nu).max(1e-6).powf(1.5))
}

impl Model {
    /// The densities (Rayleigh, Mie, ozone) at height `h` km.
    fn density(&self, h: f32) -> Vec3 {
        Vec3::new(
            (-h / self.hr).exp(),
            (-h / self.hm).exp(),
            (1.0 - (h - OZONE_CENTER).abs() / OZONE_HALF_WIDTH).max(0.0),
        )
    }

    /// Scattering (Rayleigh, Mie) and extinction at density `d`.
    fn coefficients(&self, d: Vec3) -> (Vec3, f32, Vec3) {
        let sr = self.rayleigh * d.x;
        let sm = self.mie * d.y;
        let ext = sr + Vec3::splat(self.mie_ext * d.y) + self.ozone * d.z;
        (sr, sm, ext)
    }

    /// The sunlight reaching radius `r` with the sun at zenith cosine `mu`:
    /// the planet's shadow (softened over the sun's half-degree) times the
    /// air's transmittance.
    fn sun_transmittance(&self, r: f32, mu: f32) -> Vec3 {
        // The horizon's zenith cosine at r.
        let horizon = -(1.0 - (self.bottom / r).min(1.0).powi(2)).max(0.0).sqrt();
        let lit = smoothstep(horizon - 0.006, horizon + 0.003, mu);
        if lit <= 0.0 {
            return Vec3::ZERO;
        }
        // Past the horizon the Chapman column's tangent identity is exact
        // only for rays that miss the ground: hold it at the horizon.
        let mu = mu.max(horizon + 0.003);
        let depth = self.rayleigh * (self.hr * chapman(r, mu, self.hr, self.bottom))
            + Vec3::splat(self.mie_ext * self.hm * chapman(r, mu, self.hm, self.bottom))
            + self.ozone * ozone_column(r, mu, self.bottom);
        (-depth).exp() * lit
    }

    /// Integrates along a ray from `o` (km, planet-centred) in unit
    /// direction `d`, the sun along unit `sun` with illuminance 1, at
    /// `steps` samples distributed quadratically (dense near the viewer).
    /// Returns (in-scattered radiance, transmittance, the transfer of an
    /// isotropic unit radiance `f_ms` sees). With `ms`: Hillaire's
    /// multiple-scattering term from the table; without: the phase is
    /// isotropic (the table's own integrand).
    fn march(
        &self,
        o: Vec3,
        d: Vec3,
        sun: Vec3,
        steps: u32,
        ms: Option<&MsTable>,
    ) -> (Vec3, Vec3, Vec3) {
        let Some(top) = sphere(o, d, self.top) else {
            return (Vec3::ZERO, Vec3::ONE, Vec3::ZERO);
        };
        let ground = sphere(o, d, self.bottom).filter(|g| g.x > 0.0);
        let start = top.x.max(0.0);
        let end = ground.map_or(top.y, |g| g.x);
        if end <= start {
            return (Vec3::ZERO, Vec3::ONE, Vec3::ZERO);
        }
        let nu = d.dot(sun);
        let (pr, pm) = match ms {
            Some(_) => (rayleigh_phase(nu), mie_phase(nu, self.g)),
            None => (0.25 / PI, 0.25 / PI),
        };
        let len = end - start;
        let n = steps as f32;
        let mut l = Vec3::ZERO;
        let mut f = Vec3::ZERO;
        let mut t = Vec3::ONE;
        for i in 0..steps {
            let (a, b) = ((i as f32 / n).powi(2), ((i + 1) as f32 / n).powi(2));
            let dt = (b - a) * len;
            let p = o + d * (start + (a + b) * 0.5 * len);
            let r = p.length();
            let h = r - self.bottom;
            let mu = p.dot(sun) / r;
            let (sr, sm, ext) = self.coefficients(self.density(h));
            let ts = self.sun_transmittance(r, mu);
            let mut s = (sr * pr + Vec3::splat(sm * pm)) * ts;
            if let Some(ms) = ms {
                s += (sr + Vec3::splat(sm)) * ms.at(h, mu);
            }
            let seg = (-ext * dt).exp();
            let ext = ext.max(Vec3::splat(1e-7));
            // Energy-conserving integration over the segment (Hillaire 2015).
            l += t * (s - s * seg) / ext;
            f += t * ((sr + Vec3::splat(sm)) - (sr + Vec3::splat(sm)) * seg) / ext;
            t *= seg;
        }
        if let Some(g) = ground {
            let p = o + d * g.x;
            let n = p / p.length();
            let mu = n.dot(sun);
            l += t * self.sun_transmittance(self.bottom, mu) * mu.max(0.0) * self.albedo / PI;
        }
        (l, t, f)
    }

    /// Hillaire's multiple-scattering table: Ψ_ms at [`MS_HEIGHTS`] for
    /// [`MS_ENTRIES`] sun zenith cosines from -1 to 1, each the second
    /// order (64 directions, isotropic phase) over 1 − f_ms.
    fn ms_table(&self) -> MsTable {
        let mut v = [Vec3::ZERO; 2 * MS_ENTRIES];
        const DIRS: u32 = 64;
        // A spherical Fibonacci set of directions.
        let dirs: Vec<Vec3> = (0..DIRS)
            .map(|i| {
                let y = 1.0 - (i as f32 + 0.5) / DIRS as f32 * 2.0;
                let rr = (1.0 - y * y).max(0.0).sqrt();
                let phi = i as f32 * 2.399_963;
                Vec3::new(rr * phi.cos(), y, rr * phi.sin())
            })
            .collect();
        for (k, &hk) in MS_HEIGHTS.iter().enumerate() {
            let o = Vec3::new(0.0, self.bottom + hk.max(0.001), 0.0);
            for j in 0..MS_ENTRIES {
                let mu = -1.0 + 2.0 * j as f32 / (MS_ENTRIES - 1) as f32;
                let sun = Vec3::new((1.0 - mu * mu).max(0.0).sqrt(), mu, 0.0);
                let (mut l2, mut fms) = (Vec3::ZERO, Vec3::ZERO);
                for &d in &dirs {
                    let (l, _, f) = self.march(o, d, sun, 20, None);
                    l2 += l;
                    fms += f;
                }
                // The averages over the sphere times the isotropic phase's
                // integral (1): ∮ x p_u dω = mean(x).
                l2 /= DIRS as f32;
                fms /= DIRS as f32;
                v[k * MS_ENTRIES + j] = l2 / (Vec3::ONE - fms).max(Vec3::splat(1e-3));
            }
        }
        MsTable(v)
    }
}

/// Ψ_ms at the two heights over the sun's zenith cosine.
#[derive(Clone)]
struct MsTable([Vec3; 2 * MS_ENTRIES]);

impl MsTable {
    /// Ψ_ms at height `h` km and sun zenith cosine `mu`, as the shader
    /// reads it: linear in mu, linear in height up to the second height.
    fn at(&self, h: f32, mu: f32) -> Vec3 {
        let x = (mu.clamp(-1.0, 1.0) * 0.5 + 0.5) * (MS_ENTRIES - 1) as f32;
        let i = (x.floor() as usize).min(MS_ENTRIES - 2);
        let f = x - i as f32;
        let row = |k: usize| self.0[k * MS_ENTRIES + i].lerp(self.0[k * MS_ENTRIES + i + 1], f);
        let w = ((h - MS_HEIGHTS[0]) / (MS_HEIGHTS[1] - MS_HEIGHTS[0])).clamp(0.0, 1.0);
        row(0).lerp(row(1), w)
    }
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The physical sky's scattering (`sky_physical.rs`), its block named
/// `SCATTER` until [`sky_scatter_wgsl`] names it.
const SKY_SCATTER_WGSL: &str = wgsl!("sky_scatter.wgsl");

/// The physical sky's functions reading their block from `block` (the
/// frame's `frame.sky_scatter`, the image-based light's `params.scatter`).
pub(crate) fn sky_scatter_wgsl(block: &str) -> String {
    SKY_SCATTER_WGSL.replace("SCATTER[", &format!("{block}["))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn luma(c: Vec3) -> f32 {
        c.dot(Vec3::new(0.2126, 0.7152, 0.0722))
    }

    fn sun_at(elevation_deg: f32) -> Vec3 {
        let e = elevation_deg.to_radians();
        Vec3::new(e.cos(), e.sin(), 0.0)
    }

    /// The Chapman approximation against a brute-force column through the
    /// same exponential layer, from the ground and from 10 km, at the
    /// zenith, the horizon and just past it: within 4 %. (The sky's colour
    /// at sunset hangs on the grazing column.)
    #[test]
    fn the_chapman_column_matches_a_numeric_one() {
        let bottom = 6360.0;
        for h in [8.0f32, 1.2] {
            for alt in [0.0f32, 10.0] {
                for mu in [1.0f32, 0.5, 0.1, 0.0, -0.02] {
                    let r = bottom + alt;
                    let o = Vec3::new(0.0, r, 0.0);
                    let d = Vec3::new((1.0 - mu * mu).sqrt(), mu, 0.0);
                    if mu < 0.0 && sphere(o, d, bottom).is_some_and(|g| g.y > 0.0) {
                        continue;
                    }
                    let (mut sum, n, len) = (0.0f64, 200_000, 3000.0f64);
                    for i in 0..n {
                        let t = (i as f64 + 0.5) / n as f64 * len;
                        let p = o.as_dvec3() + d.as_dvec3() * t;
                        sum += (-(p.length() - bottom as f64) / h as f64).exp() * len / n as f64;
                    }
                    let approx = (chapman(r, mu, h, bottom) * h) as f64;
                    let err = (approx - sum).abs() / sum;
                    assert!(
                        err < 0.04,
                        "H {h} alt {alt} mu {mu}: {approx} vs {sum} ({err})"
                    );
                }
            }
        }
    }

    /// The sun's light: white-ish at noon, red at sunset, gone once the
    /// planet hides it; and ozone and haze only ever take light away.
    #[test]
    fn the_sun_reddens_towards_the_horizon() {
        let a = PhysicalAtmosphere::EARTH;
        let noon = a.sun_transmittance(0.0, sun_at(60.0).y);
        assert!(noon.min_element() > 0.6 && noon.x > noon.z, "{noon}");
        let low = a.sun_transmittance(0.0, sun_at(2.0).y);
        assert!(
            low.x > 2.0 * low.y && low.y > 2.0 * low.z,
            "sunset is red: {low}"
        );
        assert!(low.x < noon.x);
        assert_eq!(a.sun_transmittance(0.0, sun_at(-3.0).y), Vec3::ZERO);
        // A viewer 10 km up still sees the sun 2° below their horizontal.
        assert!(a.sun_transmittance(10_000.0, sun_at(-2.0).y).x > 0.0);
        let hazy = PhysicalAtmosphere { mie: 40.0, ..a };
        assert!(
            hazy.sun_transmittance(0.0, sun_at(20.0).y).x
                < a.sun_transmittance(0.0, sun_at(20.0).y).x
        );
    }

    /// The clear noon sky: blue overhead (b > g > r), paler towards the
    /// horizon, its zenith below sunlit grey ground's brightness, brightest
    /// towards the sun (the aerosols' glow), the ground below darker.
    #[test]
    fn the_noon_sky_is_blue_and_brightens_towards_the_horizon_and_the_sun() {
        let a = PhysicalAtmosphere::EARTH;
        let sun = sun_at(60.0);
        let sky = |d: Vec3| a.radiance(0.0, d.normalize(), sun, 64);
        let zenith = sky(Vec3::Y);
        assert!(zenith.z > zenith.y && zenith.y > zenith.x, "{zenith}");
        // Sunlit grey ground (albedo 0.3) under this sun at the day-night
        // noon's 3: 0.3 * 3 * cos 30° / π ≈ 0.25. The zenith is a third
        // to two thirds of it.
        let l = luma(zenith) * 1.1;
        assert!((0.08..0.17).contains(&l), "zenith luma {l}");
        let horizon = sky(Vec3::new(-1.0, 0.05, 0.0));
        assert!(
            horizon.z / horizon.x < zenith.z / zenith.x,
            "paler: {horizon} vs {zenith}"
        );
        assert!(luma(horizon) > luma(zenith));
        let near_sun = sky(sun + Vec3::new(0.0, 0.0, 0.08));
        assert!(luma(near_sun) > 2.0 * luma(zenith), "{near_sun}");
        let ground = sky(Vec3::new(0.0, -0.5, 1.0));
        assert!(luma(ground) < luma(horizon));
    }

    /// Sunset: the sky towards the sun glows orange (red over blue), the
    /// zenith stays blue (ozone), the whole sky dims as the sun sinks and
    /// is near black at -12° (astronomical dark lies past the floor).
    #[test]
    fn the_sky_warms_and_darkens_through_twilight() {
        let a = PhysicalAtmosphere::EARTH;
        let toward = |e: f32| a.radiance(0.0, Vec3::new(1.0, 0.04, 0.0).normalize(), sun_at(e), 64);
        let zenith = |e: f32| a.radiance(0.0, Vec3::Y, sun_at(e), 64);
        let dusk = toward(1.0);
        assert!(dusk.x > dusk.z, "the sunset side is warm: {dusk}");
        let z = zenith(-2.0);
        assert!(z.z > z.x, "the twilight zenith is blue: {z}");
        let mut last = f32::MAX;
        for e in [30.0, 10.0, 2.0, -2.0, -6.0, -12.0] {
            let l = luma(zenith(e));
            assert!(l < last, "{e}°: {l} after {last}");
            last = l;
        }
        assert!(last < 1e-3 * luma(zenith(30.0)), "{last}");
    }

    /// Hillaire's term adds light (most where the sun is low) and is
    /// finite: f_ms stays below 1 for Earth.
    #[test]
    fn multiple_scattering_adds_light() {
        let m = PhysicalAtmosphere::EARTH.model();
        let ms = m.ms_table();
        for v in ms.0 {
            assert!(v.is_finite() && v.min_element() >= 0.0, "{v}");
        }
        let o = Vec3::new(0.0, m.bottom + 0.001, 0.0);
        let sun = sun_at(20.0);
        let single = m
            .march(
                o,
                Vec3::Y,
                sun,
                64,
                Some(&MsTable([Vec3::ZERO; 2 * MS_ENTRIES])),
            )
            .0;
        let both = m.march(o, Vec3::Y, sun, 64, Some(&ms)).0;
        assert!(both.x > single.x && both.z > single.z);
        assert!(both.z < 2.0 * single.z, "{both} vs {single}");
    }

    /// The GPU block: on, the scale in row 3, the table after six rows.
    #[test]
    fn the_gpu_block_carries_the_table() {
        let g = PhysicalAtmosphere::EARTH.gpu();
        assert_eq!(g[0][3], 1.0);
        assert_eq!(g[3][3], SKY_SCALE);
        assert!((g[3][0] - 6360.0).abs() < 1e-3 && (g[3][1] - 6460.0).abs() < 1e-3);
        assert!(g[6 + MS_ENTRIES - 1][2] > 0.0);
    }
}
