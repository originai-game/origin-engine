//! Weather on the lit surfaces: wetness, puddles, snow and dust.

use glam::Vec3;

/// Weather on the lit surfaces (`shaders/weather_surface.wgsl`): how wet
/// they are, how much standing water lies on level ground and how much
/// snow covers what faces up. The runtime takes these from og.Weather's
/// state, simulated in the tick; the renderer only draws them. Every lit
/// surface takes them (standard, physical and terrain materials), placed
/// by its world position and geometric normal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceWeather {
    /// 0 dry to 1 soaked: porous surfaces darken, all of them gain a sheen.
    pub wetness: f32,
    /// 0 none to 1: how far puddles spread over level ground.
    pub puddles: f32,
    /// 0 none to 1: snow from the flattest faces (a dusting) to steep ones.
    pub snow: f32,
    /// How hard it is raining now, which ripples the puddles (0: still).
    pub rain: f32,
    /// Linear snow colour.
    pub snow_color: Vec3,
    /// The size of the puddles' pattern, in metres.
    pub puddle_size: f32,
    /// Which way the rain or snow falls (normalised; straight down by
    /// default): the shelter map looks along it, so a wind that slants the
    /// rain wets under the lee of an eave less than under its windward side.
    pub fall: Vec3,
    /// The side, in metres, of the square around the camera in which
    /// roofs and overhangs keep what is under them dry and free of snow
    /// (the weather occlusion map, [`shelter_view`]); 0: nothing is
    /// sheltered. Past it every surface is open to the sky.
    pub shelter: f32,
    /// 0 none to 1: dust and sand settled on what faces up, filling the
    /// low parts first (a dust storm leaves it, rain washes it off).
    pub dust: f32,
    /// Linear colour of the settled dust.
    pub dust_color: Vec3,
    /// The depth of lying snow, in metres: a terrain's surface is lifted
    /// by it where the snow covers it (drawn only).
    pub snow_depth: f32,
}

impl Default for SurfaceWeather {
    fn default() -> Self {
        SurfaceWeather {
            wetness: 0.0,
            puddles: 0.0,
            snow: 0.0,
            rain: 0.0,
            snow_color: Vec3::new(0.9, 0.93, 0.97),
            puddle_size: 3.0,
            fall: Vec3::NEG_Y,
            shelter: 64.0,
            dust: 0.0,
            dust_color: Vec3::new(0.62, 0.5, 0.36),
            snow_depth: 0.0,
        }
    }
}

impl SurfaceWeather {
    /// These surfaces under the weather `look`: how hard it rains on them
    /// now and which way it falls, slanted by the wind at most 45 degrees
    /// (what drifts flatter eddies round a roof, not under it). What lies
    /// on them (wetness, puddles, snow, dust) stays: it is the state the
    /// weather left, not the weather.
    pub fn under(self, look: &crate::WeatherLook) -> SurfaceWeather {
        let down = (-look.fall.y).max(0.0);
        let drift = Vec3::new(look.fall.x, 0.0, look.fall.z).clamp_length_max(down);
        SurfaceWeather {
            rain: look.rain,
            fall: (drift - Vec3::Y * down).normalize_or(Vec3::NEG_Y),
            ..self
        }
    }

    /// The frame uniform's rows (`frame.weather`, `frame.weather_look`,
    /// `frame.weather_dust`, `frame.weather_depth`).
    pub(crate) fn gpu(&self) -> [[f32; 4]; 4] {
        // (The shelter map's rows are the renderer's: its view and the
        // cascade array's layer it lies in, `AtmosphereGpu::shelter`.)
        [
            [
                self.wetness.clamp(0.0, 1.0),
                self.puddles.clamp(0.0, 1.0),
                self.snow.clamp(0.0, 1.0),
                self.rain.clamp(0.0, 2.0),
            ],
            self.snow_color
                .max(Vec3::ZERO)
                .extend(self.puddle_size.max(0.05))
                .to_array(),
            self.dust_color
                .max(Vec3::ZERO)
                .extend(self.dust.clamp(0.0, 1.0))
                .to_array(),
            [self.snow_depth.clamp(0.0, 4.0), 0.0, 0.0, 0.0],
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{WEATHER_DUST, WEATHER_RAIN, WEATHER_SNOW, weather};

    fn under(kind: u32, intensity: f32, wind: Vec3) -> SurfaceWeather {
        let lying = SurfaceWeather {
            wetness: 0.4,
            snow: 0.3,
            ..SurfaceWeather::default()
        };
        lying.under(&weather(kind, intensity, wind, 18.0, Vec3::ONE, 1.0, None))
    }

    /// The rain falls at 11 m/s and the snow at 1.1: a 5.5 m/s wind slants
    /// the rain a little under 27 degrees and would lay the snow nearly
    /// flat, which is held at 45. Only the rain ripples the puddles, a
    /// rising wind slants nothing, dust falls straight, and what lies on
    /// the surfaces is not the weather's to change.
    #[test]
    fn surfaces_take_the_rain_and_the_fall_of_the_weather_over_them() {
        let close = |a: Vec3, b: Vec3| (a - b).length() < 1e-4;
        let rain = under(WEATHER_RAIN, 1.5, Vec3::new(5.5, 3.0, 0.0));
        assert_eq!(rain.rain, 1.5);
        let s = 5f32.sqrt();
        assert!(
            close(rain.fall, Vec3::new(1.0 / s, -2.0 / s, 0.0)),
            "{:?}",
            rain.fall
        );
        assert_eq!((rain.wetness, rain.snow), (0.4, 0.3));

        let snow = under(WEATHER_SNOW, 1.0, Vec3::new(4.0, 0.0, -3.0));
        assert_eq!(snow.rain, 0.0);
        let h = 0.5f32.sqrt();
        assert!(
            close(snow.fall, Vec3::new(0.8 * h, -h, -0.6 * h)),
            "{:?}",
            snow.fall
        );

        let dust = under(WEATHER_DUST, 2.0, Vec3::new(9.0, 0.0, 0.0));
        assert_eq!((dust.rain, dust.fall), (0.0, Vec3::NEG_Y));
    }
}
