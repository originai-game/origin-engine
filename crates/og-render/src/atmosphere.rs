//! Atmosphere (the E5 round): fog, a procedural sky with a chosen look, and
//! the frame-uniform block both read. Everything is off by default: a scene
//! without fog and with the built-in sky draws exactly the frame it drew
//! before (the shaders branch on a flag and return their input).
//!
//! - [`Fog`]: distance fog (exponential density from `start`, or a linear
//!   ramp from `start` to `end`) and exponential height fog (dense below
//!   `height`, thinning by `falloff` per metre above it), mixed towards a
//!   colour that turns towards `sun_color` looking into the sun (forward
//!   scattering), capped at `max_opacity`. Applied in every lit, sky,
//!   particle, strip and world-sprite shader of the main pass; the overlay
//!   is not fogged.
//! - [`SkyLook`]: the procedural sky's colours (zenith, horizon, ground),
//!   sun disc and glow, stars at night and brightness, for
//!   [`crate::ProceduralSky::look`] (the image-based light) and
//!   [`crate::Background::Procedural`] (the sky drawn per pixel, so a moving
//!   sun moves smoothly while the lighting is regenerated in steps).

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
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
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
    /// intensity, glow), glow power and sun colour, then the sun direction.
    pub sky: [[f32; 4]; 6],
    /// x: seconds (tick / rate), y: the tick, z: ticks per second.
    pub time: [f32; 4],
}

impl AtmosphereGpu {
    /// `sky`: the drawn sky's look, sun direction and sun colour.
    pub(crate) fn new(
        fog: Option<&Fog>,
        sky: Option<(&SkyLook, Vec3, Vec3)>,
        tick: u64,
        rate: f32,
    ) -> AtmosphereGpu {
        let mut a = AtmosphereGpu::default();
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
            a.sky[5] = sun.normalize_or(Vec3::Y).extend(0.0).to_array();
        }
        let rate = if rate > 0.0 { rate } else { 60.0 };
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
    /// A fog colour that matches the horizon.
    pub fog_color: Vec3,
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
        mix3(Vec3::new(0.10, 0.12, 0.32), Vec3::new(0.18, 0.36, 0.78), day),
        Vec3::new(0.004, 0.006, 0.02),
        night,
    );
    let horizon = mix3(
        mix3(Vec3::new(0.98, 0.52, 0.26), Vec3::new(0.78, 0.84, 0.92), day),
        Vec3::new(0.02, 0.03, 0.07),
        night,
    );
    let ground = mix3(
        mix3(Vec3::new(0.14, 0.09, 0.08), Vec3::new(0.16, 0.14, 0.12), day),
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
        (-sun, Vec3::new(0.55, 0.65, 1.0), 0.45 * ramp(0.5, 1.0, night))
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
    };
    DayNightLook {
        sun_direction: sun,
        light_direction,
        light_color,
        light_intensity,
        sky,
        sun_color,
        ambient: 0.12 + 0.88 * (0.25 * golden + 0.75 * day),
        fog_color: horizon * (0.8 + 0.2 * day),
    }
}
