//! Atmosphere (the E5 round): fog, a procedural sky with a chosen look, and
//! the frame-uniform block both read. Everything is off by default: a scene
//! without fog and with the built-in sky draws exactly the frame it drew
//! before (the shaders branch on a flag and return their input).
//!
//! - [`Fog`]: distance fog measured from the camera (exponential density
//!   past `start` and, when `end > start`, a linear ramp from `start` to
//!   full fog at `end`, the two multiplied together) and exponential height fog (dense below
//!   `height`, thinning by `falloff` per metre above it), mixed towards a
//!   colour that turns towards `sun_color` looking into the sun (forward
//!   scattering), capped at `max_opacity`. Applied in every lit, sky,
//!   particle, strip and world-sprite shader of the main pass; the overlay
//!   is not fogged.
//! - [`SkyLook`]: the procedural sky's colours (zenith, horizon, ground),
//!   sun disc and glow, stars at night, a moon disc with its phase, and
//!   brightness, for
//!   [`crate::ProceduralSky::look`] (the image-based light) and
//!   [`crate::Background::Procedural`] (the sky drawn per pixel, so a moving
//!   sun moves smoothly; the lighting follows it through the live sky's
//!   capture, `sky_capture.rs`).

use glam::Vec3;

/// Fog over the scene. See the module documentation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fog {
    /// Linear colour the fog tends to (HDR, before exposure).
    pub color: Vec3,
    /// Exponential distance fog density per metre (0: none).
    pub density: f32,
    /// Where distance fog starts, in metres from the camera.
    pub start: f32,
    /// With `end > start`, a linear ramp from `start` to `end` as well.
    pub end: f32,
    /// Height fog density at and below `height` (0: none).
    pub height_density: f32,
    /// The world height below which the height fog is densest.
    pub height: f32,
    /// How fast height fog thins above `height`, per metre.
    pub falloff: f32,
    /// The fog's colour looking into the sun (forward scattering).
    pub sun_color: Vec3,
    /// How tight the sun's scattering lobe is (higher: smaller).
    pub sun_power: f32,
    /// The most the fog hides (1: completely at distance).
    pub max_opacity: f32,
    /// How much of the fog covers the sky (0: the sky stays clear).
    pub sky: f32,
}

impl Default for Fog {
    fn default() -> Self {
        Fog {
            color: Vec3::new(0.62, 0.68, 0.76),
            density: 0.02,
            start: 0.0,
            end: 0.0,
            height_density: 0.0,
            height: 0.0,
            falloff: 0.5,
            sun_color: Vec3::new(1.0, 0.86, 0.62),
            sun_power: 8.0,
            max_opacity: 1.0,
            sky: 1.0,
        }
    }
}

/// The look of the procedural sky. The built-in look (no `SkyLook`) is the
/// one every scene had before, drawn by the same code.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkyLook {
    /// Linear colour overhead.
    pub zenith: Vec3,
    /// Linear colour at the horizon.
    pub horizon: Vec3,
    /// Linear colour below the horizon.
    pub ground: Vec3,
    /// The sun disc's angular radius in degrees (0: no disc).
    pub sun_size: f32,
    /// The disc's brightness (multiplies the sun colour).
    pub sun_intensity: f32,
    /// The glow around the sun.
    pub glow: f32,
    /// How tight the glow is.
    pub glow_power: f32,
    /// Stars: 0 none, 1 a full night sky (they fade where the sky is bright).
    pub stars: f32,
    /// A multiplier on the whole sky.
    pub brightness: f32,
    /// The moon disc's angular radius in degrees (0: no moon). It sets
    /// below the horizon and hides the stars behind it.
    pub moon_size: f32,
    /// How much of the moon is lit: 0 new, 0.25 the first quarter (its
    /// right half lit), 0.5 full, 0.75 the last quarter (its left half).
    pub moon_phase: f32,
    /// The lit moon's brightness (multiplies `moon_color`).
    pub moon_intensity: f32,
    /// The moon's linear colour.
    pub moon_color: Vec3,
    /// Towards the moon; zero: opposite the drawn sun.
    pub moon_direction: Vec3,
    /// Draw the sky by scattering sunlight through this atmosphere instead
    /// of the gradient (`zenith`, `horizon`, `ground` and `glow` unused;
    /// the sun disc is reddened by the air; `brightness`, the stars and the
    /// moon apply as before). `None`: the gradient.
    pub physical: Option<crate::PhysicalAtmosphere>,
}

impl Default for SkyLook {
    /// The built-in sky's numbers, as a starting point.
    fn default() -> Self {
        SkyLook::DEFAULT
    }
}

impl SkyLook {
    /// The built-in sky's numbers ([`Default`]).
    pub const DEFAULT: SkyLook = SkyLook {
        zenith: Vec3::new(0.18, 0.36, 0.78),
        horizon: Vec3::new(0.78, 0.84, 0.92),
        ground: Vec3::new(0.16, 0.14, 0.12),
        sun_size: 1.8,
        sun_intensity: 40.0,
        glow: 0.6,
        glow_power: 24.0,
        stars: 0.0,
        brightness: 1.1,
        moon_size: 0.0,
        moon_phase: 0.5,
        moon_intensity: 1.4,
        moon_color: Vec3::new(0.9, 0.92, 1.0),
        moon_direction: Vec3::ZERO,
        physical: None,
    };
    /// The cosines of the sun disc's outer and inner edge.
    pub(crate) fn disc(&self) -> (f32, f32) {
        let r = self.sun_size.clamp(0.0, 20.0).to_radians();
        // A soft edge two fifths of the radius wide.
        ((r * 1.2).cos(), (r * 0.8).cos())
    }

    /// The look as five vec4s (frame.wgsl's sky_* and ibl.wgsl's look),
    /// with the sun's colour in the last.
    pub(crate) fn gpu(&self, on: bool, sun_color: Vec3) -> [[f32; 4]; 5] {
        let (outer, inner) = self.disc();
        [
            self.zenith.extend(f32::from(on)).to_array(),
            self.horizon.extend(self.brightness).to_array(),
            self.ground.extend(self.stars).to_array(),
            [outer, inner, self.sun_intensity, self.glow],
            [self.glow_power, sun_color.x, sun_color.y, sun_color.z],
        ]
    }
}

/// The frame uniform's atmosphere block (`frame.wgsl`): fog, the sky look
/// the sky pass draws per pixel, the clock.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct AtmosphereGpu {
    /// rgb: colour, w: 1 when fog is on.
    pub fog_color: [f32; 4],
    /// x: density, y: start, z: end, w: max opacity.
    pub fog_distance: [f32; 4],
    /// x: height, y: falloff, z: height density, w: sky amount.
    pub fog_height: [f32; 4],
    /// rgb: sun scattering colour, w: power.
    pub fog_sun: [f32; 4],
    /// zenith (w: 1 when the sky pass draws the look), horizon (w:
    /// brightness), ground (w: stars), disc (outer cos, inner cos,
    /// intensity, glow), glow power and sun colour, then the sun direction,
    /// the moon's direction (w: its angular radius in radians, 0 none) and
    /// its lit colour (w: the phase).
    pub sky: [[f32; 4]; 8],
    /// x: seconds (tick / rate), y: the tick, z: ticks per second.
    pub time: [f32; 4],
    /// The physical sky's block (`PhysicalAtmosphere::gpu`; zero: the
    /// gradient), read by `sky_scatter.wgsl` as `frame.sky_scatter`.
    pub scatter: [[f32; 4]; crate::sky_physical::SCATTER_VEC4S],
    /// The clouds' block (`Clouds::gpu`; zero: none), `frame.clouds`.
    pub clouds: [[f32; 4]; crate::clouds::CLOUD_VEC4S],
    /// The aurora's block (`Aurora::gpu`; zero: none), `frame.aurora`.
    pub aurora: [[f32; 4]; crate::aurora::AURORA_VEC4S],
    /// Weather on the lit surfaces (`SurfaceWeather::gpu`; zero: dry).
    pub weather: [[f32; 4]; 2],
    /// The weather occlusion map (`weather::shelter_view`): its view
    /// projection, then x: its layer of the cascade array plus one (0: no
    /// map; the layer is always the array's last, so cascades or cached
    /// layers added in front of it move it, never a constant), y: the drip
    /// line's soft edge in texels, z: the map's side in texels, w: the
    /// depth bias. Written by the renderer after `new`.
    pub shelter: [[f32; 4]; 5],
}

impl AtmosphereGpu {
    /// `sky`: the drawn sky's look, sun direction and sun colour. `clouds`:
    /// with whether the live sky captures them (RenderScene::sky_captured).
    pub(crate) fn new(
        fog: Option<&Fog>,
        sky: Option<(&SkyLook, Vec3, Vec3)>,
        clouds: Option<(&crate::Clouds, bool)>,
        aurora: Option<&crate::Aurora>,
        weather: Option<&crate::SurfaceWeather>,
        tick: u64,
        rate: f32,
    ) -> AtmosphereGpu {
        let mut a: AtmosphereGpu = bytemuck::Zeroable::zeroed();
        if let Some(f) = fog {
            a.fog_color = f.color.extend(1.0).to_array();
            a.fog_distance = [
                f.density.max(0.0),
                f.start.max(0.0),
                f.end.max(0.0),
                f.max_opacity.clamp(0.0, 1.0),
            ];
            a.fog_height = [
                f.height,
                f.falloff.max(1e-4),
                f.height_density.max(0.0),
                f.sky.clamp(0.0, 1.0),
            ];
            a.fog_sun = f.sun_color.extend(f.sun_power.max(0.0)).to_array();
        }
        if let Some((look, sun, color)) = sky {
            let g = look.gpu(true, color);
            a.sky[..5].copy_from_slice(&g);
            let sun = sun.normalize_or(Vec3::Y);
            a.sky[5] = sun.extend(0.0).to_array();
            let size = look.moon_size.clamp(0.0, 20.0);
            if size > 0.0 {
                let moon = look.moon_direction.normalize_or(-sun);
                a.sky[6] = moon.extend(size.to_radians()).to_array();
                a.sky[7] = (look.moon_color * look.moon_intensity.max(0.0))
                    .extend(look.moon_phase.rem_euclid(1.0))
                    .to_array();
            }
            if let Some(p) = &look.physical {
                a.scatter = p.gpu();
            }
        }
        let rate = if rate > 0.0 { rate } else { 60.0 };
        if let Some(w) = weather {
            a.weather = w.gpu();
        }
        // Clouds and the aurora belong to the procedural sky the sky pass
        // draws per pixel.
        if let Some((look, ..)) = sky {
            let radius = look
                .physical
                .map_or(6_360_000.0, |p| p.planet_radius.clamp(10.0, 1.0e5) * 1000.0);
            if let Some((c, live)) = clouds {
                a.clouds = c.gpu(tick as f64 / f64::from(rate), radius);
                // The clouds' ambient is the clear sky's, never the image-based
                // light's: the live sky captures the clouds into it, and lit by
                // their own capture they converged several times too dark (an
                // overcast noon drawn as night).
                let (look, sun, color) = sky.expect("clouds only with the sky");
                let row = crate::clouds::CLEAR_SKY_ROW;
                for (i, e) in clear_sky_light(look, sun, color).iter().enumerate() {
                    a.clouds[row + i] = e.extend(0.0).to_array();
                }
                // A live sky holds the deck in the image-based light, which the
                // scene then leaves undimmed (RenderScene::sky_captured): none
                // of the sky light's dimming for the clouds to undo.
                if live {
                    a.clouds[4][2] = 1.0;
                }
            }
            if let Some(au) = aurora {
                a.aurora = au.gpu(radius * 0.001);
            }
        }
        // Seconds wrap every 2^16 ticks so f32 keeps sub-tick precision.
        a.time = [(tick % 65536) as f32 / rate, tick as f32, rate, 0.0];
        a
    }
}

/// What a time of day looks like: the sun (or at night the moon) as the
/// directional light, the sky, the image-based light's strength and a fog
/// tint, from [`day_night`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DayNightLook {
    /// Towards the sun (below the horizon at night).
    pub sun_direction: Vec3,
    /// Towards the light that lights the scene: the sun by day, the moon
    /// (opposite the sun) by night.
    pub light_direction: Vec3,
    /// The directional light's linear colour and intensity.
    pub light_color: Vec3,
    pub light_intensity: f32,
    /// The sky to draw and to light with.
    pub sky: SkyLook,
    /// The sun colour the sky and its light use.
    pub sun_color: Vec3,
    /// The image-based light's intensity.
    pub ambient: f32,
    /// The part of `ambient` that is the eye's adaptation to the twilight's
    /// shade, not the sky's light (`RenderScene::adaptation`): 1 by day.
    pub adaptation: f32,
    /// A fog colour that matches the horizon.
    pub fog_color: Vec3,
    /// The fog's colour looking into the sun (`Fog::sun_color`), when the
    /// look knows it: the physical sky's horizon under the sun (dim and red
    /// once the sun has set). `None`: the fog keeps its own.
    pub fog_sun: Option<Vec3>,
}

impl DayNightLook {
    /// The same time of day under a physical sky: the sky scattered
    /// through `air` (its sun disc reddened by it), the sun's light its
    /// colour after crossing the air to a viewer `air.altitude` up (gone
    /// once the planet hides it, when the moon takes over as before), the
    /// image-based light at full strength (the scattered sky carries the
    /// dimming), and a fog colour of the sky's horizon. A pure function of
    /// its inputs, like [`day_night`].
    pub fn with_atmosphere(mut self, air: &crate::PhysicalAtmosphere) -> DayNightLook {
        // The sun above the air: a 5800 K white.
        let white = Vec3::new(1.0, 0.97, 0.93);
        let through = air.sun_transmittance(air.altitude, self.sun_direction.y);
        self.sun_color = white;
        self.sky.physical = Some(*air);
        self.sky.sun_intensity = 40.0;
        self.sky.glow = 0.0;
        // Brighter than the gradient's zenith blue with the default
        // exposure, as the gradient's brightness was.
        self.sky.brightness = 1.1;
        if self.light_direction == self.sun_direction {
            self.light_color = (white * through).max(Vec3::splat(1e-4));
            // The gradient's light ramps with the elevation; the air does
            // that now, so the light is the sun's at full strength scaled to
            // the gradient's noon (3) and fades only past the horizon.
            let k = (through.dot(Vec3::new(0.2126, 0.7152, 0.0722))).max(1e-4);
            self.light_color /= k;
            self.light_intensity = 3.0 * k;
        }
        // The image-based light at full strength by day, and up to four
        // times it as the sun sets: the eye's adaptation to the shade (as
        // an exposure opened for the land, not the sky), so land under a
        // twilight sky keeps a deep blue-grey instead of falling into the
        // tone curve's toe (its scene light is already a few percent of
        // noon's; the curve maps that to black).
        self.adaptation = 1.0 + 3.0 * (1.0 - ramp(-0.1, 0.15, self.sun_direction.y));
        self.ambient = self.adaptation;
        // The fog's colour: the light the air scatters towards a point, the
        // sky's mean over the upper hemisphere (cosine-weighted, three
        // bearings at three heights), not its bright horizon.
        let s = self.sun_direction;
        let side = Vec3::new(-s.z, 0.0, s.x).normalize_or(Vec3::X);
        let toward = Vec3::new(s.x, 0.0, s.z).normalize_or(Vec3::Z);
        let (mut sum, mut weight) = (Vec3::ZERO, 0.0);
        for bearing in [side, -side, toward] {
            for y in [0.1f32, 0.4, 0.8] {
                let d = (bearing * (1.0 - y * y).sqrt() + Vec3::Y * y).normalize();
                sum += air.radiance(air.altitude, d, s, 12) * y;
                weight += y;
            }
        }
        let horizon = sum / weight;
        self.fog_color = (horizon * white * 1.1 + air.night).max(Vec3::splat(1e-4));
        // Looking into the sun the fog takes the sky's glow on the horizon
        // under it, not a fixed warm white: after sunset that is dim and
        // red, so the fog no longer shines over the darkened land.
        let glow = air.radiance(air.altitude, (toward + Vec3::Y * 0.05).normalize(), s, 12);
        self.fog_sun = Some((glow * white + air.night).max(Vec3::splat(1e-4)));
        self
    }
}

/// The clear sky's radiance along unit `d` as the sky pass draws it under
/// `look` (frame.wgsl `sky_look`, sky.wgsl `sky_physical_look`) without the
/// sun's disc, the stars or the moon: the gradient or the scattered air,
/// times the look's brightness.
fn clear_sky(look: &SkyLook, sun: Vec3, color: Vec3, d: Vec3) -> Vec3 {
    if let Some(p) = &look.physical {
        return (p.radiance(p.altitude, d, sun, 8) * color + p.night.max(Vec3::ZERO))
            * look.brightness;
    }
    let up = d.y.clamp(-1.0, 1.0);
    let sky = mix3(look.horizon, look.zenith, up.max(0.0).powf(0.55));
    let sky = mix3(look.ground, sky, ramp(-0.08, 0.02, up));
    let glow = d.dot(sun).max(0.0).powf(look.glow_power.max(1.0)) * look.glow;
    sky * look.brightness + color * glow
}

/// The clear sky's light on surfaces facing overhead, then horizontally
/// towards the sun's bearing, away from it and across it: the cosine
/// weighted mean of [`clear_sky`] over each hemisphere (the irradiance over
/// π, as the image-based light holds it), by a fixed quadrature of three
/// rings of eight. What the clouds are lit by besides the sun (frame.wgsl
/// `clear_sky_light`): the sky above and around them, which holds no
/// clouds.
pub(crate) fn clear_sky_light(look: &SkyLook, sun: Vec3, color: Vec3) -> [Vec3; 4] {
    let s = sun.normalize_or(Vec3::Y);
    let toward = Vec3::new(s.x, 0.0, s.z).normalize_or(Vec3::Z);
    let across = Vec3::new(-toward.z, 0.0, toward.x);
    [Vec3::Y, toward, -toward, across].map(|n| {
        let t = if n.y.abs() > 0.5 { Vec3::X } else { Vec3::Y };
        let b = n.cross(t).normalize();
        let t = b.cross(n);
        let (mut sum, mut weight) = (Vec3::ZERO, 0.0);
        for theta in [15.0f32, 45.0, 75.0] {
            let (st, ct) = theta.to_radians().sin_cos();
            for k in 0..8 {
                let (sp, cp) = (k as f32 * std::f32::consts::FRAC_PI_4).sin_cos();
                let d = (n * ct + (t * cp + b * sp) * st).normalize();
                let w = ct * st;
                sum += clear_sky(look, s, color, d) * w;
                weight += w;
            }
        }
        sum / weight
    })
}

fn mix3(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

fn ramp(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The look of `hours` (0..24; 6 sunrise, 12 noon, 18 sunset) for a sun
/// whose path is tilted `tilt` degrees from overhead towards -z and turned
/// `azimuth` degrees about +y (0: it rises in +x). A pure function of its
/// inputs, so a day-night clock driven by the tick looks the same on every
/// run and after a restore.
pub fn day_night(hours: f32, tilt: f32, azimuth: f32) -> DayNightLook {
    let a = (hours.rem_euclid(24.0) - 6.0) / 24.0 * std::f32::consts::TAU;
    let (tilt, azimuth) = (tilt.to_radians(), azimuth.to_radians());
    let path = Vec3::new(a.cos(), a.sin() * tilt.cos(), -a.sin() * tilt.sin());
    let sun = glam::Quat::from_rotation_y(azimuth) * path;
    let e = sun.y;
    // Palettes by the sun's elevation: night, twilight, golden hour, day.
    let day = ramp(0.05, 0.45, e);
    let golden = ramp(-0.12, 0.06, e);
    let night = 1.0 - ramp(-0.22, -0.04, e);
    let zenith = mix3(
        mix3(
            Vec3::new(0.10, 0.12, 0.32),
            Vec3::new(0.18, 0.36, 0.78),
            day,
        ),
        Vec3::new(0.004, 0.006, 0.02),
        night,
    );
    let horizon = mix3(
        mix3(
            Vec3::new(0.98, 0.52, 0.26),
            Vec3::new(0.78, 0.84, 0.92),
            day,
        ),
        Vec3::new(0.02, 0.03, 0.07),
        night,
    );
    let ground = mix3(
        mix3(
            Vec3::new(0.14, 0.09, 0.08),
            Vec3::new(0.16, 0.14, 0.12),
            day,
        ),
        Vec3::new(0.005, 0.005, 0.01),
        night,
    );
    let sun_color = mix3(
        Vec3::new(1.0, 0.42, 0.16),
        Vec3::new(1.0, 0.95, 0.88),
        ramp(0.0, 0.35, e),
    );
    let moon = night > 0.5;
    let (light_direction, light_color, light_intensity) = if moon {
        (
            -sun,
            Vec3::new(0.55, 0.65, 1.0),
            0.45 * ramp(0.5, 1.0, night),
        )
    } else {
        (sun, sun_color, 3.0 * golden * (0.35 + 0.65 * day))
    };
    let sky = SkyLook {
        zenith,
        horizon,
        ground,
        sun_size: 1.8,
        sun_intensity: 40.0 * golden,
        glow: 0.6 + 0.6 * (1.0 - day) * golden,
        glow_power: 24.0 - 14.0 * (1.0 - day),
        stars: night,
        brightness: 1.1,
        ..SkyLook::DEFAULT
    };
    DayNightLook {
        sun_direction: sun,
        light_direction,
        light_color,
        light_intensity,
        sky,
        sun_color,
        ambient: 0.12 + 0.88 * (0.25 * golden + 0.75 * day),
        adaptation: 1.0,
        fog_color: horizon * (0.8 + 0.2 * day),
        fog_sun: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The clouds' clear-sky light is the sky's cosine-weighted mean: a
    /// uniform sky's own radiance on every face, and at a physical sunset
    /// brighter facing the sun's bearing than away from it (the glow the
    /// cloud bases take), overhead brighter than a noon's horizon is dark.
    #[test]
    fn the_clear_sky_light_is_the_skys_mean() {
        let flat = SkyLook {
            zenith: Vec3::splat(0.4),
            horizon: Vec3::splat(0.4),
            ground: Vec3::splat(0.4),
            glow: 0.0,
            ..SkyLook::DEFAULT
        };
        for e in clear_sky_light(&flat, Vec3::new(0.3, 0.5, -0.8), Vec3::ONE) {
            assert!(
                (e - Vec3::splat(0.4 * 1.1)).abs().max_element() < 1e-5,
                "{e}"
            );
        }
        let air = crate::PhysicalAtmosphere::EARTH;
        let dusk = day_night(17.7, 25.0, -20.0).with_atmosphere(&air);
        let [_, toward, away, _] = clear_sky_light(&dusk.sky, dusk.sun_direction, dusk.sun_color);
        let luma = |v: Vec3| v.dot(Vec3::new(0.2126, 0.7152, 0.0722));
        println!("dusk: toward {toward}, away {away}");
        assert!(luma(toward) > 1.2 * luma(away), "{toward} vs {away}");
        let noon = day_night(12.5, 25.0, -20.0).with_atmosphere(&air);
        let [up, ..] = clear_sky_light(&noon.sky, noon.sun_direction, noon.sun_color);
        println!("noon overhead {up}");
        assert!(luma(up) > 0.05, "{up}");
    }

    /// The sun's direct light on level ground against the sky's, the
    /// contrast its shadows have there.
    fn ground_contrast(look: &DayNightLook) -> f32 {
        look.light_intensity * look.light_direction.y.max(0.0) / look.ambient
    }

    /// og-taste §8's warm dusk: at the default tilt (25°) the sun is 10 to
    /// 20° up between 16.5 and 17.25 h, warm from about 17 h, and its light
    /// on level ground still at least half the sky's, so shadows read. By
    /// 17.5 h it is under 7° and its light a third of the sky's (the
    /// shadows fade), and at 18 h it sets: the path is the equinox's,
    /// whatever the tilt, which only lowers the noon sun.
    #[test]
    fn the_dusk_hours_keep_sun_shadows() {
        let elevation = |h: f32| day_night(h, 25.0, 0.0).sun_direction.y.asin().to_degrees();
        for h in [16.5, 17.0, 17.25] {
            let look = day_night(h, 25.0, 0.0);
            assert!(
                ground_contrast(&look) >= 0.5,
                "{h} h: {}",
                ground_contrast(&look)
            );
            assert!(
                (10.0..=21.0).contains(&elevation(h)),
                "{h} h: {}°",
                elevation(h)
            );
        }
        for h in [17.0, 17.25] {
            let c = day_night(h, 25.0, 0.0).light_color;
            assert!(c.z < 0.75 * c.x, "{h} h: the sun is warm, {c}");
        }
        assert!(ground_contrast(&day_night(17.5, 25.0, 0.0)) < 0.4);
        assert!(elevation(17.5) < 7.0);
        assert!(elevation(18.0).abs() < 0.01);
        // Tilt lowers the path, never the hour of sunset.
        assert!(day_night(18.0, 0.0, 0.0).sun_direction.y.abs() < 1e-4);
        assert!(
            elevation(17.0)
                < day_night(17.0, 0.0, 0.0)
                    .sun_direction
                    .y
                    .asin()
                    .to_degrees()
        );
    }

    /// The day-night look under the physical sky: the sun's light is the
    /// air's colour (warm and dimmer at 17:30 than at noon, none once it has
    /// set, the moon taking over later as before), the sky is drawn
    /// physically, and the fog is the sky's mean colour, darker at dusk.
    #[test]
    fn the_physical_day_night_light_crosses_the_air() {
        let air = crate::PhysicalAtmosphere::EARTH;
        let at = |h: f32| day_night(h, 25.0, 0.0).with_atmosphere(&air);
        let (noon, dusk, set) = (at(12.0), at(17.5), at(18.3));
        assert!(noon.sky.physical.is_some() && noon.ambient == 1.0);
        let lum = |c: Vec3| c.dot(Vec3::new(0.2126, 0.7152, 0.0722));
        let warm = |c: Vec3| c.x / c.z;
        assert!(warm(dusk.light_color) > 2.0 * warm(noon.light_color));
        assert!(dusk.light_intensity < 0.5 * noon.light_intensity);
        assert!(set.light_intensity < 1e-3, "{}", set.light_intensity);
        assert_eq!(at(23.0).light_direction, -at(23.0).sun_direction);
        assert!(lum(dusk.fog_color) < 0.5 * lum(noon.fog_color));
        assert!(noon.fog_color.z > noon.fog_color.x, "{}", noon.fog_color);
    }
}
