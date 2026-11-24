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
    /// How much of the fog covers the sky (0: the sky stays clear). What
    /// the sky draws under the horizon stands for land and takes all of it.
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
    /// The physical sky's sun disc: the sun's angular radius seen from the
    /// Earth, its mean semi-diameter (16′), in degrees.
    pub const PHYSICAL_SUN_SIZE: f32 = 0.2666;
    /// The physical sky's disc brightness: the light (radiance times area)
    /// the 0.8° disc it replaced drew at 40, which bloom spreads into the
    /// glow round the sun; a disc a third as wide at 40 left it a speck
    /// with none.
    pub const PHYSICAL_SUN_INTENSITY: f32 = 360.0;

    /// The moon's light above the air as the physical sky scatters it, in
    /// the sky's units (the sun's is its colour at 1): the full moon's
    /// share of the sun's, the share og.DayNight's moonlight has of its
    /// sun, by its phase ([`moon_phase_light`]) and as bright as the drawn
    /// moon against its default. Zero without a moon.
    pub fn moon_light(&self) -> Vec3 {
        if self.moon_size <= 0.0 {
            return Vec3::ZERO;
        }
        self.moon_color.max(Vec3::ZERO) * self.moon_light_share()
    }

    /// [`SkyLook::moon_light`] over `moon_color`.
    fn moon_light_share(&self) -> f32 {
        PHYSICAL_MOONLIGHT / PHYSICAL_SUNLIGHT
            * moon_phase_light(self.moon_phase)
            * self.moon_intensity.max(0.0)
            / SkyLook::DEFAULT.moon_intensity
    }

    /// Towards the moon: `moon_direction`, or opposite `sun`.
    pub(crate) fn moon_toward(&self, sun: Vec3) -> Vec3 {
        self.moon_direction.normalize_or(-sun.normalize_or(Vec3::Y))
    }

    /// The cosines of the sun disc's outer and inner edge.
    pub(crate) fn disc(&self) -> (f32, f32) {
        let r = self.sun_size.clamp(0.0, 20.0).to_radians();
        // A soft edge two fifths of the radius wide.
        ((r * 1.2).cos(), (r * 0.8).cos())
    }

    /// The look as five vec4s (sky_look.wgsl's sky_* and ibl.wgsl's look),
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
    /// rgb: colour, w: 1 when fog is on, 2 under water
    /// ([`AtmosphereGpu::underwater`]).
    pub fog_color: [f32; 4],
    /// x: density, y: start, z: end, w: max opacity.
    pub fog_distance: [f32; 4],
    /// x: height, y: falloff, z: height density, w: sky amount.
    pub fog_height: [f32; 4],
    /// rgb: sun scattering colour, w: power.
    pub fog_sun: [f32; 4],
    /// zenith (w: 1 when the sky pass draws the look), horizon (w:
    /// brightness), ground (w: stars), disc (outer cos, inner cos,
    /// intensity, glow), glow power and sun colour, then the sun direction
    /// (w: the moon's light above the air over its lit colour,
    /// SkyLook::moon_light), the moon's direction (w: its angular radius
    /// in radians, 0 none) and its lit colour (w: the phase).
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
    pub weather: [[f32; 4]; 4],
    /// The weather occlusion map (`weather::shelter_view`): its view
    /// projection, then x: its layer of the cascade array plus one (0: no
    /// map; the layer is always the array's last, so cascades or cached
    /// layers added in front of it move it, never a constant), y: the drip
    /// line's soft edge in texels, z: the map's side in texels, w: the
    /// depth bias. Written by the renderer after `new`.
    pub shelter: [[f32; 4]; 5],
}

impl AtmosphereGpu {
    /// The block's fog: `f`, the plain fog its shaders mix in.
    pub(crate) fn fog(&mut self, f: &Fog) {
        self.fog_color = f.color.extend(1.0).to_array();
        self.fog_distance = [
            f.density.max(0.0),
            f.start.max(0.0),
            f.end.max(0.0),
            f.max_opacity.clamp(0.0, 1.0),
        ];
        self.fog_height = [
            f.height,
            f.falloff.max(1e-4),
            f.height_density.max(0.0),
            f.sky.clamp(0.0, 1.0),
        ];
        self.fog_sun = f.sun_color.extend(f.sun_power.max(0.0)).to_array();
    }

    /// `sky`: the drawn sky's look, sun direction and sun colour. `clouds`:
    /// with whether the live sky captures them (RenderScene::sky_captured).
    /// `eye`: the camera's x and z in the world, which the clouds stand over.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        fog: Option<&Fog>,
        sky: Option<(&SkyLook, Vec3, Vec3)>,
        clouds: Option<(&crate::Clouds, bool)>,
        aurora: Option<&crate::Aurora>,
        weather: Option<&crate::SurfaceWeather>,
        tick: u64,
        rate: f32,
        eye: glam::DVec2,
    ) -> AtmosphereGpu {
        let mut a: AtmosphereGpu = bytemuck::Zeroable::zeroed();
        if let Some(f) = fog {
            a.fog(f);
        }
        if let Some((look, sun, color)) = sky {
            let g = look.gpu(true, color);
            a.sky[..5].copy_from_slice(&g);
            let sun = sun.normalize_or(Vec3::Y);
            a.sky[5] = sun.extend(0.0).to_array();
            let size = look.moon_size.clamp(0.0, 20.0);
            if size > 0.0 {
                let moon = look.moon_toward(sun);
                // w: the moon's light above the air over its lit colour
                // (SkyLook::moon_light), which the physical sky scatters.
                a.sky[5][3] = look.moon_light_share() / look.moon_intensity.max(1e-6);
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
        let radius = sky
            .and_then(|(look, ..)| look.physical)
            .map_or(6_360_000.0, |p| p.planet_radius.clamp(10.0, 1.0e5) * 1000.0);
        if let Some((c, live)) = clouds {
            a.clouds = c.gpu(tick as f64 / f64::from(rate), radius, eye);
            // Cloud heights and noise remain world-relative. The shell's
            // reference radius is world y 0, not sea level; carry that
            // offset too for sunlight and the true ground intersection.
            let altitude = sky
                .and_then(|(look, ..)| look.physical)
                .map_or(0.0, |p| p.altitude.max(0.0));
            a.clouds[4][0] += altitude;
            if let Some((look, sun, color)) = sky {
                // The clouds' ambient is the clear sky's, never the image-based
                // light's: the live sky captures the clouds into it, and lit by
                // their own capture they converged several times too dark (an
                // overcast noon drawn as night).
                let row = crate::clouds::CLEAR_SKY_ROW;
                for (i, e) in clear_sky_light(look, sun, color).iter().enumerate() {
                    a.clouds[row + i] = e.extend(0.0).to_array();
                }
                // What lies under the horizon, in the w of the three
                // horizontal rows (sky_look.wgsl `low_sky_light`).
                let under = clear_sky_mean(look, sun, color, Vec3::NEG_Y);
                for (i, e) in under.to_array().iter().enumerate() {
                    a.clouds[row + 1 + i][3] = *e;
                }
                // A live sky holds the deck in the image-based light, which the
                // scene then leaves undimmed (RenderScene::sky_captured). The
                // sea's foam takes that captured light, not the analytical
                // deck over it (water.wgsl).
                a.clouds[4][3] = f32::from(live);
            }
            // After the clear sky's rows, whose w it shares (clouds.rs `gpu`).
            a.clouds[crate::clouds::CLEAR_SKY_ROW][3] = altitude;
        }
        if sky.is_some()
            && let Some(au) = aurora
        {
            a.aurora = au.gpu(radius * 0.001);
        }
        // Seconds wrap every 3600 s (exact integer period matching
        // WIND_TIME_PERIOD) in f64 so f32 keeps sub-millisecond precision
        // without fractional phase jumps on periodic effects.
        let seconds = (tick as f64 / f64::from(rate)).rem_euclid(3600.0) as f32;
        a.time = [seconds, tick as f32, rate, 0.0];
        a
    }

    /// Makes the fog the underwater view's (media.wgsl `underwater_light`):
    /// `light` the surface's height, the daylight's attenuation per metre
    /// of depth and the light shafts' strength
    /// ([`crate::water::underwater_light`]).
    pub(crate) fn underwater(&mut self, light: [f32; 3]) {
        self.fog_color[3] = 2.0;
        self.fog_height = [light[0], light[1], 0.0, 0.0];
        self.fog_sun[3] = light[2];
    }

    /// The share of `p`'s light the fog lets through to a perspective eye at
    /// `eye`: 1 less media.wgsl `fog_amount(p)`'s amount (1 without fog).
    pub(crate) fn fog_clear(&self, eye: Vec3, p: Vec3) -> f32 {
        if self.fog_color[3] < 0.5 {
            return 1.0;
        }
        let to = p - eye;
        let d = to.length();
        let fd = self.fog_distance;
        let mut clear = (-fd[0] * (d - fd[1]).max(0.0)).exp();
        if fd[2] > fd[1] {
            clear *= 1.0 - ((d - fd[1]) / (fd[2] - fd[1])).clamp(0.0, 1.0);
        }
        let fh = self.fog_height;
        if fh[2] > 0.0 {
            let k = fh[1];
            let base = fh[2] * (-k * (eye.y - fh[0])).exp();
            let column = if (k * to.y).abs() > 1e-4 {
                base * (1.0 - (-k * to.y).exp()) / (k * to.y) * d
            } else {
                base * d
            };
            clear *= (-column.max(0.0)).exp();
        }
        1.0 - (1.0 - clear).min(fd[3])
    }

    /// The share of the sky at `dir` the fog leaves (sky.wgsl `sky_fog`:
    /// the fog 5 km out, times the fog's sky amount): what the fog leaves
    /// of the sun's disc.
    pub(crate) fn sky_fog_clear(&self, eye: Vec3, dir: Vec3) -> f32 {
        if self.fog_color[3] < 0.5 {
            return 1.0;
        }
        let amount = 1.0 - self.fog_clear(eye, eye + dir * 5000.0);
        1.0 - amount * self.fog_height[3]
    }
}

/// What a time of day looks like: the sun (or at night the moon) as the
/// directional light, the sky, the image-based light's strength and a fog
/// tint, from [`day_night`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DayNightLook {
    /// Towards the sun (below the horizon at night).
    pub sun_direction: Vec3,
    /// The sun's light and the moon's, each by its own height: the two
    /// candidates the key light ([`crate::key_light`]) draws one of, handing
    /// over between them at dusk and dawn ([`DayNightLook::lights`]).
    pub sun_light: BodyLight,
    pub moon_light: BodyLight,
    /// The sky to draw and to light with.
    pub sky: SkyLook,
    /// The sun colour the sky and its light use.
    pub sun_color: Vec3,
    /// The image-based light's intensity.
    pub ambient: f32,
    /// The part of `ambient` that is the eye's adaptation to the twilight's
    /// and the night's shade, not the sky's light
    /// (`RenderScene::adaptation`): 1 by day.
    pub adaptation: f32,
    /// A fog colour that matches the horizon.
    pub fog_color: Vec3,
    /// The fog's colour looking into the sun (`Fog::sun_color`), when the
    /// look knows it: the physical sky's horizon under the sun (dim and red
    /// once the sun has set). `None`: the fog keeps its own.
    pub fog_sun: Option<Vec3>,
}

/// A sky body's light on the land: towards it, its linear colour and its
/// intensity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BodyLight {
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: f32,
}

impl DayNightLook {
    /// The sun's and the moon's lights as directional lights with `base`'s
    /// shadow settings, their intensity times `scale`: the candidates the
    /// key light takes for og.DayNight.
    pub fn lights(
        &self,
        base: &crate::DirectionalLight,
        scale: f32,
    ) -> [crate::DirectionalLight; 2] {
        [self.sun_light, self.moon_light].map(|b| crate::DirectionalLight {
            direction_to_light: b.direction,
            color: b.color,
            intensity: b.intensity * scale,
            ..*base
        })
    }

    /// The light the key light draws of the two alone, with the default
    /// shadow settings.
    pub fn key(&self) -> crate::DirectionalLight {
        let lights = self.lights(&crate::DirectionalLight::default(), 1.0);
        crate::key_light(&lights).map_or(lights[0], |k| k.light)
    }

    /// The moon `size` degrees in radius at `phase` (as [`SkyLook`]'s),
    /// opposite the sun: drawn in the sky, scattered by the air under a
    /// physical sky ([`SkyLook::moon_light`]), and the moonlight's
    /// strength by its phase ([`moon_phase_light`]: none at new moon). Before
    /// [`DayNightLook::with_atmosphere`], whose fog takes it.
    pub fn with_moon(mut self, size: f32, phase: f32) -> DayNightLook {
        self.sky.moon_size = size;
        self.sky.moon_phase = phase;
        self.lit()
    }

    /// The sun's light and the moon's, each its own light by its own
    /// height: the sun's by [`day_night_under`]'s ramps, or under a
    /// physical sky its colour after crossing the air; the moon's
    /// ([`MOONLIGHT`] by its phase) full from about 13° up under the
    /// gradient, and under a physical sky crossing the air as the sun's
    /// does, low and orange as it rises. The moon stands opposite the sun,
    /// so it rises as the sun sets and takes the light over a few minutes
    /// after sunset, not once the twilight has gone. Which of them is drawn
    /// is the key light's choice ([`crate::key_light`]), which dims the
    /// light through the swap where the two are about equal, so the shadows
    /// never jump from one side to the other at full strength.
    fn lit(mut self) -> DayNightLook {
        let sun = self.sun_direction;
        let moon = self.sky.moon_toward(sun);
        let phase = moon_phase_light(self.sky.moon_phase);
        let lum = |c: Vec3| c.dot(Vec3::new(0.2126, 0.7152, 0.0722));
        let ((sun_color, sun_light), (moon_color, moon_light)) = match &self.sky.physical {
            None => {
                let e = sun.y;
                let light = 3.0 * ramp(-0.12, 0.06, e) * (0.35 + 0.65 * ramp(0.05, 0.45, e));
                (
                    (self.sun_color, light),
                    (MOON_COLOR, MOONLIGHT * phase * ramp(0.0, 0.22, moon.y)),
                )
            }
            Some(air) => {
                let through = |y: f32| air.sun_transmittance(air.altitude, y);
                // The sun's light at full strength scaled to the gradient's
                // noon (3): the air ramps it with the elevation and ends it
                // past the horizon.
                let t = through(sun.y);
                let k = lum(t);
                let sun_color = (self.sun_color * t).max(Vec3::splat(1e-4)) / k.max(1e-4);
                // The moon's likewise, as bright as PHYSICAL_MOONLIGHT
                // overhead and its colour there the night's blue.
                let (tm, top) = (through(moon.y), through(1.0));
                let km = lum(tm);
                let moon_color = if km > 1e-6 {
                    (MOON_COLOR * tm / top) * (lum(top) / km)
                } else {
                    MOON_COLOR
                };
                (
                    (sun_color, PHYSICAL_SUNLIGHT * k),
                    (moon_color, PHYSICAL_MOONLIGHT * phase * km / lum(top)),
                )
            }
        };
        self.sun_light = BodyLight {
            direction: sun,
            color: sun_color,
            intensity: sun_light,
        };
        self.moon_light = BodyLight {
            direction: moon,
            color: moon_color,
            intensity: moon_light,
        };
        self
    }

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
        self.sun_color = white;
        self.sky.physical = Some(*air);
        self.sky.sun_size = SkyLook::PHYSICAL_SUN_SIZE;
        self.sky.sun_intensity = SkyLook::PHYSICAL_SUN_INTENSITY;
        self.sky.glow = 0.0;
        // Brighter than the gradient's zenith blue with the default
        // exposure, as the gradient's brightness was.
        self.sky.brightness = 1.1;
        // The sun's and the moon's light cross the air.
        self = self.lit();
        // The image-based light at full strength by day, and up to four
        // times it as the sun sets: the eye's adaptation to the shade (as
        // an exposure opened for the land, not the sky), so land under a
        // twilight sky keeps a deep blue-grey instead of falling into the
        // tone curve's toe (its scene light is already a few percent of
        // noon's; the curve maps that to black).
        // The fog's colour: the light the air scatters towards a point, the
        // sky's mean over the upper hemisphere (cosine-weighted, three
        // bearings at three heights), not its bright horizon.
        let s = self.sun_direction;
        let side = Vec3::new(-s.z, 0.0, s.x).normalize_or(Vec3::X);
        let toward = Vec3::new(s.x, 0.0, s.z).normalize_or(Vec3::Z);
        // The moon's light scattered too, where it shines.
        let (moon, moon_light) = (self.sky.moon_toward(s), self.sky.moon_light());
        let (mut sum, mut moonlit, mut weight) = (Vec3::ZERO, Vec3::ZERO, 0.0);
        for bearing in [side, -side, toward] {
            for y in [0.1f32, 0.4, 0.8] {
                let d = (bearing * (1.0 - y * y).sqrt() + Vec3::Y * y).normalize();
                sum += air.radiance(air.altitude, d, s, 12) * white * y;
                if moon_light != Vec3::ZERO {
                    moonlit += air.radiance(air.altitude, d, moon, 12) * moon_light * y;
                }
                weight += y;
            }
        }
        let horizon = (sum + moonlit) / weight;
        // The image-based light at full strength by day, and up to four
        // times it as the sun sets: the eye's adaptation to the shade (as
        // an exposure opened for the land, not the sky), so land under a
        // twilight sky keeps a deep blue-grey instead of falling into the
        // tone curve's toe (its scene light is already a few percent of
        // noon's; the curve maps that to black). The moon's light in the
        // sky fills that shade: the eye adapts to what the sun and the
        // night leave of it, less as the moonlit sky outshines them (a
        // full moon high up, about none), so the land in the moon's shade
        // stays below the land in its light.
        let lum = |c: Vec3| c.dot(Vec3::new(0.2126, 0.7152, 0.0722));
        let unlit = lum(sum / weight + air.night.max(Vec3::ZERO));
        let shade = unlit / (unlit + lum(moonlit / weight)).max(1e-9);
        self.adaptation = 1.0 + 3.0 * (1.0 - ramp(-0.1, 0.15, self.sun_direction.y)) * shade;
        self.ambient = self.adaptation;
        self.fog_color = (horizon * 1.1 + air.night).max(Vec3::splat(1e-4));
        // Looking into the sun the fog takes the sky's glow on the horizon
        // under it, not a fixed warm white: after sunset that is dim and
        // red, so the fog no longer shines over the darkened land.
        let glow = air.radiance(air.altitude, (toward + Vec3::Y * 0.05).normalize(), s, 12);
        self.fog_sun = Some((glow * white + air.night).max(Vec3::splat(1e-4)));
        self
    }
}

/// The clear sky's radiance along unit `d` as the sky pass draws it under
/// `look` (sky_look.wgsl `sky_look`, sky.wgsl `sky_physical_look`) without the
/// sun's disc, the stars or the moon's disc: the gradient or the air's
/// scattering of the sun and the moon, times the look's brightness.
fn clear_sky(look: &SkyLook, sun: Vec3, color: Vec3, d: Vec3) -> Vec3 {
    if let Some(p) = &look.physical {
        let moon = look.moon_light();
        let moonlit = if moon == Vec3::ZERO {
            Vec3::ZERO
        } else {
            p.radiance(p.altitude, d, look.moon_toward(sun), 8) * moon
        };
        return (p.radiance(p.altitude, d, sun, 8) * color + moonlit + p.night.max(Vec3::ZERO))
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
/// rings of eight. What the clouds are lit by besides the sun (sky_look.wgsl
/// `clear_sky_light`): the sky above and around them, which holds no
/// clouds.
pub(crate) fn clear_sky_light(look: &SkyLook, sun: Vec3, color: Vec3) -> [Vec3; 4] {
    let s = sun.normalize_or(Vec3::Y);
    let toward = Vec3::new(s.x, 0.0, s.z).normalize_or(Vec3::Z);
    let across = Vec3::new(-toward.z, 0.0, toward.x);
    [Vec3::Y, toward, -toward, across].map(|n| clear_sky_mean(look, sun, color, n))
}

/// The cosine weighted mean of [`clear_sky`] over the hemisphere about unit
/// `n` ([`clear_sky_light`]). About straight down it is what lies under
/// the horizon: the ground the sky draws there, which a surface facing
/// the horizon takes half of its light from (sky_look.wgsl
/// `low_sky_light` takes it out again).
fn clear_sky_mean(look: &SkyLook, sun: Vec3, color: Vec3, n: Vec3) -> Vec3 {
    let s = sun.normalize_or(Vec3::Y);
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
}

fn mix3(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

fn ramp(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The moon's light at full night, a third of the noon sun's: the night's
/// key light, so moonlit stone and paths stand brighter than the sky
/// behind them.
const MOONLIGHT: f32 = 1.0;

/// The moonlight's linear colour by night (overhead under a physical sky).
const MOON_COLOR: Vec3 = Vec3::new(0.55, 0.65, 1.0);

/// The moon's light under a physical sky ([`DayNightLook::with_atmosphere`]),
/// whose nights are lit in the air's own scale and read through the
/// camera's exposure.
const PHYSICAL_MOONLIGHT: f32 = 0.45;

/// The sun's light at full strength under a physical sky
/// ([`DayNightLook::with_atmosphere`]): the gradient's noon.
const PHYSICAL_SUNLIGHT: f32 = 3.0;

/// The moon's light at `phase` (0 new, 0.25 the first quarter, 0.5 full,
/// 0.75 the last quarter) over the full moon's: the light a sphere of
/// Lommel–Seeliger regolith sends back at the phase angle φ, 1 - sin(φ/2)
/// tan(φ/2) ln(cot(φ/4)) (Jensen, Durand, Stark, Premože, Dorsey and
/// Shirley, "A Physically-Based Night Sky Model", SIGGRAPH 2001, §4.1).
/// About 0.38 at a quarter, 0 at new moon.
pub fn moon_phase_light(phase: f32) -> f32 {
    let phi = (std::f32::consts::TAU * (phase.rem_euclid(1.0) - 0.5).abs())
        .min(std::f32::consts::PI - 1e-3);
    if phi < 1e-4 {
        return 1.0;
    }
    let h = 0.5 * phi;
    (1.0 - h.sin() * h.tan() * (1.0 / (0.25 * phi).tan()).ln()).clamp(0.0, 1.0)
}

/// The least light the gradient sky gives the land ([`sky_luminance`] times
/// the image-based light's strength): about what it gives at sunset, a
/// fifth of noon's. A night's shade holds here, in the night sky's blue,
/// so stone out of the moon's light still reads.
const SHADE_FLOOR: f32 = 0.12;

/// The mean luminance of the gradient sky `look` draws, between its
/// horizon and its zenith: what its image-based light gives the land.
fn sky_luminance(look: &SkyLook) -> f32 {
    ((look.zenith + look.horizon) * 0.5).dot(Vec3::new(0.2126, 0.7152, 0.0722)) * look.brightness
}

/// The look of `hours` (0..24; 6 sunrise, 12 noon, 18 sunset) for a sun
/// that rises at the heading `azimuth` (degrees about +y from +z towards
/// +x, `og_core::facing`'s: 90 rises in +x) and whose path is tilted `tilt`
/// degrees from overhead towards the heading `azimuth` + 90. A pure function of its
/// inputs, so a day-night clock driven by the tick looks the same on every
/// run and after a restore.
pub fn day_night(hours: f32, tilt: f32, azimuth: f32) -> DayNightLook {
    let a = (hours.rem_euclid(24.0) - 6.0) / 24.0 * std::f32::consts::TAU;
    let tilt = tilt.to_radians();
    // The path of a sun rising in +x; a heading counts from +z, a quarter
    // turn before it.
    let path = Vec3::new(a.cos(), a.sin() * tilt.cos(), -a.sin() * tilt.sin());
    day_night_under(glam::Quat::from_rotation_y((azimuth - 90.0).to_radians()) * path)
}

/// The look under a sun towards unit `sun`, wherever a clock, a date or a
/// latitude puts it: [`day_night`]'s palettes, light and sky by the sun's
/// height alone. A pure function of its input.
pub fn day_night_under(sun: Vec3) -> DayNightLook {
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
    let sky = SkyLook {
        zenith,
        horizon,
        ground,
        sun_size: 1.8,
        sun_intensity: 40.0 * golden,
        // Gone with the twilight: the glow is drawn about the sun wherever
        // it is, and a sun deep under the horizon would shine up through
        // the ground into the image-based light.
        glow: (0.6 + 0.6 * (1.0 - day) * golden) * (1.0 - night),
        glow_power: 24.0 - 14.0 * (1.0 - day),
        stars: night,
        brightness: 1.1,
        ..SkyLook::DEFAULT
    };
    // The sky's light on the land: the image-based light's strength by the
    // clock, times the sky it is made from. From sunset on it would fall to
    // a few thousandths of noon's, which the tone curve draws as black, so
    // the eye adapts and holds it at the floor.
    let sky_light = 0.12 + 0.88 * (0.25 * golden + 0.75 * day);
    let shade = sky_light * sky_luminance(&sky);
    let adaptation = (SHADE_FLOOR / shade.max(1e-6)).max(1.0);
    DayNightLook {
        sun_direction: sun,
        // Set by `lit`.
        sun_light: BodyLight {
            direction: sun,
            color: sun_color,
            intensity: 0.0,
        },
        moon_light: BodyLight {
            direction: -sun,
            color: MOON_COLOR,
            intensity: 0.0,
        },
        sky,
        sun_color,
        ambient: sky_light * adaptation,
        adaptation,
        fog_color: horizon * (0.8 + 0.2 * day),
        fog_sun: None,
    }
    .lit()
}
