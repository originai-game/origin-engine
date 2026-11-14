//! Weather presets (the E5 round): rain, snow, dust and mist as a particle
//! emitter, a fog, the cloud cover they fall from and a dimming of the
//! light, tuned by intensity, wind and the size of the region around the
//! camera they fill. Intensity is continuous: it changes the particles'
//! rate, never their capacity, so a weather that builds or clears keeps
//! the particles in the air (a changed capacity restarts a system).

use crate::{
    Background, Clouds, ColorCurve, Curve, EmitterShape, Fog, ParticleAlign, ParticleBlend,
    ParticleEmitter, ParticleNoise, ParticleRender, SimulationSpace,
};
use glam::Vec3;

pub use og_env::weather::{
    DUST as WEATHER_DUST, MIST as WEATHER_MIST, NONE as WEATHER_NONE, RAIN as WEATHER_RAIN,
    SNOW as WEATHER_SNOW,
};

/// What a weather preset adds to a frame.
#[derive(Clone, Debug, PartialEq)]
pub struct WeatherLook {
    /// The particles (`None` for mist and clear weather), to be placed at
    /// [`WeatherLook::offset`] from the camera.
    pub emitter: Option<ParticleEmitter>,
    /// Where the emitter sits relative to the camera, before
    /// [`WeatherLook::ahead`].
    pub offset: Vec3,
    /// How far along the camera's level forward direction the region is
    /// centred (half its size: what is behind the camera is not seen).
    pub ahead: f32,
    /// Its fog.
    pub fog: Option<Fog>,
    /// A multiplier on the directional light (and half as much on the
    /// ambient), applied with [`crate::DirectionalLight::cover`], which
    /// softens and fills in the sun's shadow by what it scatters.
    /// [`WeatherLook::light_scene`] applies it under the clouds.
    pub light: f32,
    /// The cloud cover this weather falls from (rain and snow: the sky
    /// closes as they build, by `dim`), 0 for what hangs in the air under
    /// any sky (dust, mist).
    pub cover: f32,
    /// How hard it rains (its intensity; 0 for every other weather): what
    /// ripples the puddles under it ([`crate::SurfaceWeather::under`]).
    pub rain: f32,
    /// The velocity what falls settles at, in m/s: the rain's or the
    /// snow's fall slanted by the level wind, straight down for what does
    /// not fall. The weather's shelter looks along it
    /// ([`crate::SurfaceWeather::under`]).
    pub fall: Vec3,
}

/// The greatest intensity: what each emitter's rate and capacity are set
/// for, the intensity being the share of it born
/// ([`ParticleEmitter::emission`]).
const FULL: f32 = 2.0;

/// The most particles of each preset in the air.
const RAIN_CAPACITY: f32 = 40_000.0;
const SNOW_CAPACITY: f32 = 56_000.0;
const DUST_CAPACITY: f32 = 8_000.0;

/// An emitter's rate and capacity for `rate` particles a second living
/// `life` seconds with at most `most` in the air: a region wider than
/// that fills is thinner, never one whose particles are overwritten
/// before they die.
fn schedule(rate: f32, life: f32, most: f32) -> (f32, u32) {
    let rate = rate.min(most / life);
    (rate, (rate * life).ceil().max(1.0) as u32)
}

/// The preset `kind` (a `WEATHER_*` code) at `intensity` (0..2), blown by
/// `wind` (m/s), filling `radius` metres around the camera, its particles
/// tinted by `tint`; `dim` scales how much it darkens the light;
/// `sprite_layer` draws the particles as 2D sprites in that layer (an
/// orthographic camera) instead of billboards.
pub fn weather(
    kind: u32,
    intensity: f32,
    wind: Vec3,
    radius: f32,
    tint: Vec3,
    dim: f32,
    sprite_layer: Option<i32>,
) -> WeatherLook {
    let i = intensity.clamp(0.0, FULL);
    let emission = i / FULL;
    let r = radius.clamp(1.0, 200.0);
    let two_d = sprite_layer.is_some();
    // In 2D the region is the xy plane in front of the camera.
    let half = |across: f32, up: f32| {
        if two_d {
            Vec3::new(across, up, 0.05)
        } else {
            Vec3::new(across, up, across)
        }
    };
    let render = match sprite_layer {
        Some(layer) => ParticleRender::Sprite { layer },
        None => ParticleRender::Billboard,
    };
    // What falls settles at its fall speed slanted by the level wind, as
    // its particles do (their drag scales both alike). The world's wind,
    // not the 2D layer's: the shelter is the world's.
    let level = Vec3::new(wind.x, 0.0, wind.z);
    let settling = move |fall: f32| level - Vec3::Y * fall;
    let wind = if two_d {
        Vec3::new(wind.x, wind.y, 0.0)
    } else {
        wind
    };
    let tinted = |c: [f32; 4]| [c[0] * tint.x, c[1] * tint.y, c[2] * tint.z, c[3]];
    let ahead = if two_d { 0.0 } else { r * 0.6 };
    let dim = dim.clamp(0.0, 1.0);
    let dimmed = |amount: f32| 1.0 - amount * (i * dim).min(1.0);
    // The sky closes over the first half of the intensity.
    let closing = |cover: f32| {
        let t = (i / 0.5).min(1.0);
        cover * t * t * (3.0 - 2.0 * t) * dim
    };
    // A particle with drag d under acceleration g settles at g / d: the
    // fall and the wind are set as that terminal velocity.
    let drifting = |fall: f32, drag: f32| (wind + Vec3::new(0.0, -fall, 0.0)) * drag;
    let (dust_rate, dust_capacity) = schedule(r * r * 0.8 * FULL, 4.0, DUST_CAPACITY);
    match kind {
        WEATHER_RAIN => {
            let fall = 11.0;
            let height = r * 1.2;
            let (rate, max_particles) = schedule(r * r * 28.0 * FULL, height / fall, RAIN_CAPACITY);
            WeatherLook {
                emitter: Some(ParticleEmitter {
                    max_particles,
                    emission,
                    // Born anywhere in the column, so the view fills at once.
                    shape: EmitterShape::Box {
                        half_extents: half(r, height * 0.5),
                    },
                    rate,
                    lifetime: [height / fall * 0.9, height / fall],
                    speed: [-fall, -fall * 0.9],
                    size: if two_d { [0.03, 0.04] } else { [0.012, 0.018] },
                    color: tinted([0.72, 0.78, 0.9, 0.5]),
                    color_over_life: ColorCurve::new(&[
                        (0.0, [1.0, 1.0, 1.0, 0.0]),
                        (0.08, [1.0, 1.0, 1.0, 1.0]),
                        (0.9, [1.0, 1.0, 1.0, 1.0]),
                        (1.0, [1.0, 1.0, 1.0, 0.0]),
                    ]),
                    gravity: drifting(fall, 1.0),
                    drag: 1.0,
                    blend: ParticleBlend::Alpha,
                    space: SimulationSpace::World,
                    align: ParticleAlign::Velocity { stretch: 0.045 },
                    render,
                    ..ParticleEmitter::default()
                }),
                offset: Vec3::new(0.0, height * 0.5 - 2.0, 0.0),
                ahead,
                fog: Some(Fog {
                    color: Vec3::new(0.46, 0.5, 0.56),
                    density: 0.03 * i,
                    start: 4.0,
                    sun_color: Vec3::new(0.6, 0.62, 0.66),
                    sky: 0.6,
                    ..Fog::default()
                }),
                light: dimmed(0.55),
                cover: closing(0.95),
                rain: i,
                fall: settling(fall),
            }
        }
        WEATHER_SNOW => {
            let fall = 1.1;
            let height = r * 1.2;
            let (rate, max_particles) = schedule(r * r * 12.0 * FULL, 7.0, SNOW_CAPACITY);
            WeatherLook {
                emitter: Some(ParticleEmitter {
                    max_particles,
                    emission,
                    shape: EmitterShape::Box {
                        half_extents: half(r, height * 0.5),
                    },
                    rate,
                    lifetime: [5.0, 7.0],
                    speed: [-fall, -fall * 0.8],
                    size: if two_d { [0.06, 0.12] } else { [0.07, 0.12] },
                    color: tinted([1.0, 1.0, 1.0, 0.9]),
                    color_over_life: ColorCurve::new(&[
                        (0.0, [1.0, 1.0, 1.0, 0.0]),
                        (0.1, [1.0, 1.0, 1.0, 1.0]),
                        (0.85, [1.0, 1.0, 1.0, 1.0]),
                        (1.0, [1.0, 1.0, 1.0, 0.0]),
                    ]),
                    gravity: drifting(fall, 0.8),
                    drag: 0.8,
                    noise: ParticleNoise {
                        strength: 1.4,
                        frequency: 0.35,
                        scroll: 0.25,
                    },
                    rotation_degrees: [0.0, 360.0],
                    angular_velocity_degrees: [-60.0, 60.0],
                    blend: ParticleBlend::Alpha,
                    space: SimulationSpace::World,
                    render,
                    ..ParticleEmitter::default()
                }),
                offset: Vec3::new(0.0, height * 0.5 - 2.0, 0.0),
                ahead,
                fog: Some(Fog {
                    color: Vec3::new(0.82, 0.85, 0.9),
                    density: 0.018 * i,
                    start: 4.0,
                    sun_color: Vec3::new(0.95, 0.95, 0.98),
                    sky: 0.7,
                    ..Fog::default()
                }),
                light: dimmed(0.35),
                cover: closing(0.9),
                rain: 0.0,
                fall: settling(fall),
            }
        }
        WEATHER_DUST => WeatherLook {
            emitter: Some(ParticleEmitter {
                max_particles: dust_capacity,
                emission,
                shape: EmitterShape::Box {
                    half_extents: half(r, if two_d { r * 0.6 } else { 1.5 }),
                },
                rate: dust_rate,
                lifetime: [2.5, 4.0],
                speed: [-0.2, 0.2],
                spread_degrees: 180.0,
                size: if two_d { [0.1, 0.35] } else { [0.15, 0.45] },
                size_over_life: Curve::new(&[(0.0, 0.6), (1.0, 1.2)]),
                color: tinted([0.78, 0.62, 0.44, 0.22]),
                color_over_life: ColorCurve::new(&[
                    (0.0, [1.0, 1.0, 1.0, 0.0]),
                    (0.3, [1.0, 1.0, 1.0, 1.0]),
                    (1.0, [1.0, 1.0, 1.0, 0.0]),
                ]),
                gravity: wind * 0.6,
                drag: 0.6,
                noise: ParticleNoise {
                    strength: 0.8,
                    frequency: 0.2,
                    scroll: 0.4,
                },
                rotation_degrees: [0.0, 360.0],
                blend: ParticleBlend::Alpha,
                space: SimulationSpace::World,
                render,
                ..ParticleEmitter::default()
            }),
            offset: if two_d {
                Vec3::ZERO
            } else {
                Vec3::new(0.0, -0.5, 0.0)
            },
            ahead,
            fog: Some(Fog {
                color: Vec3::new(0.72, 0.57, 0.4),
                density: 0.05 * i,
                height_density: 0.08 * i,
                height: 0.0,
                falloff: 0.25,
                sun_color: Vec3::new(1.0, 0.75, 0.45),
                sky: 0.85,
                ..Fog::default()
            }),
            light: dimmed(0.45),
            cover: 0.0,
            rain: 0.0,
            fall: Vec3::NEG_Y,
        },
        WEATHER_MIST => WeatherLook {
            emitter: None,
            offset: Vec3::ZERO,
            ahead: 0.0,
            fog: Some(Fog {
                color: Vec3::new(0.74, 0.77, 0.8),
                density: 0.008 * i,
                height_density: 0.35 * i,
                height: 0.6,
                falloff: 0.7,
                sky: 0.4,
                ..Fog::default()
            }),
            light: dimmed(0.25),
            cover: 0.0,
            rain: 0.0,
            fall: Vec3::NEG_Y,
        },
        _ => WeatherLook {
            emitter: None,
            offset: Vec3::ZERO,
            ahead: 0.0,
            fog: None,
            light: 1.0,
            cover: 0.0,
            rain: 0.0,
            fall: Vec3::NEG_Y,
        },
    }
}

impl WeatherLook {
    /// Lights `scene` under this weather, as every scene with a weather is
    /// lit (in place of [`Clouds::light_scene`]). The sky closes to
    /// [`WeatherLook::cover`]: the scene's clouds where they cover less,
    /// and a deck of the weather's own where a procedural sky has none
    /// (rain never falls from a clear sky). The light dims as under those
    /// clouds, and the weather's own dimming ([`WeatherLook::light`]) takes
    /// only what they left: the lesser of the two beams, not their product
    /// (rain under an overcast dimmed twice drew a noon as dusk). A scene
    /// whose background is its own picture keeps it, and its light takes
    /// the weather's dimming alone. Call once, after the scene's light and
    /// declared fog, before the weather's own fog.
    pub fn light_scene(&self, scene: &mut crate::RenderScene) {
        let procedural = matches!(
            scene.lighting.environment.background,
            Background::Procedural { .. }
        );
        if scene.atmosphere.clouds.is_none() && procedural && self.cover > 0.0 {
            scene.atmosphere.clouds = Some(Clouds {
                coverage: 0.0,
                ..Clouds::default()
            });
        }
        // What the clouds took of the sky's light, by the multiplier or in
        // the live sky's capture, is `sky` either way.
        let (sun, sky) = match &mut scene.atmosphere.clouds {
            Some(clouds) => {
                clouds.coverage = clouds.coverage.max(self.cover);
                let clouds = *clouds;
                clouds.light_scene(scene);
                clouds.light()
            }
            None => (1.0, 1.0),
        };
        scene.lighting.light.cover((self.light / sun).min(1.0));
        scene.lighting.environment.intensity *= ((0.5 + 0.5 * self.light) / sky).min(1.0);
    }

    /// Where the emitter goes for a camera at `position` looking along
    /// `forward`.
    pub fn origin(&self, position: Vec3, forward: Vec3) -> Vec3 {
        let level = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
        position + self.offset + level * self.ahead
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenderScene, clouds::DECK_BEAM};

    fn look(kind: u32, intensity: f32, dim: f32) -> WeatherLook {
        weather(kind, intensity, Vec3::ZERO, 18.0, Vec3::ONE, dim, None)
    }

    /// A weather that builds changes how many of its particles are born,
    /// never the emitter's rate or capacity (either restarts or
    /// overwrites what is in the air).
    #[test]
    fn intensity_changes_the_emission_and_nothing_else_of_the_schedule() {
        for kind in [WEATHER_RAIN, WEATHER_SNOW, WEATHER_DUST] {
            // The widest region is thinner, not overwritten.
            for radius in [18.0, 200.0] {
                let at = |i: f32| {
                    weather(kind, i, Vec3::ZERO, radius, Vec3::ONE, 1.0, None)
                        .emitter
                        .unwrap()
                };
                let (light, full) = (at(0.25), at(2.0));
                assert_eq!(light.max_particles, full.max_particles, "kind {kind}");
                assert_eq!(light.rate, full.rate, "kind {kind}");
                assert_eq!((light.emission, full.emission), (0.125, 1.0));
                let life = full.lifetime[1];
                assert!(
                    full.max_particles as f32 >= full.rate * life - 1.0,
                    "kind {kind} at {radius} m: {} for {} a second living {life} s",
                    full.max_particles,
                    full.rate
                );
            }
        }
    }

    /// Rain and snow close the sky as they build, by `dim`; dust and mist
    /// hang under any sky.
    #[test]
    fn rain_and_snow_bring_their_cover() {
        assert_eq!(look(WEATHER_RAIN, 0.0, 1.0).cover, 0.0);
        assert_eq!(look(WEATHER_RAIN, 0.25, 1.0).cover, 0.475);
        assert_eq!(look(WEATHER_RAIN, 1.0, 1.0).cover, 0.95);
        assert_eq!(look(WEATHER_SNOW, 2.0, 0.5).cover, 0.45);
        assert_eq!(look(WEATHER_RAIN, 1.0, 0.0).cover, 0.0);
        assert_eq!(look(WEATHER_DUST, 2.0, 1.0).cover, 0.0);
        assert_eq!(look(WEATHER_MIST, 2.0, 1.0).cover, 0.0);
    }

    fn sky_scene() -> RenderScene {
        let mut scene = RenderScene::new();
        scene.lighting.light.intensity = 4.0;
        scene.lighting.environment.background = Background::Procedural {
            sun_direction: Vec3::Y,
            sun_color: Vec3::ONE,
        };
        scene
    }

    /// Under a procedural sky rain brings a closed deck, and the sun is
    /// dimmed once, to the deck's beam.
    #[test]
    fn rain_closes_a_clear_procedural_sky() {
        let mut scene = sky_scene();
        look(WEATHER_RAIN, 1.0, 1.0).light_scene(&mut scene);
        assert_eq!(scene.atmosphere.clouds.unwrap().coverage, 0.95);
        let sun = scene.lighting.light.intensity;
        assert!((sun - 4.0 * DECK_BEAM).abs() < 1e-6, "sun {sun}");
    }

    /// Declared clouds keep a cover greater than the weather's, and the
    /// light takes the lesser beam of the two, not their product: dust
    /// (a beam of 0.55) under a quarter cover (0.9875) leaves 0.55.
    #[test]
    fn the_light_takes_the_lesser_beam_of_clouds_and_weather() {
        let mut scene = sky_scene();
        scene.atmosphere.clouds = Some(Clouds {
            coverage: 0.25,
            ..Clouds::default()
        });
        look(WEATHER_DUST, 1.0, 1.0).light_scene(&mut scene);
        assert_eq!(scene.atmosphere.clouds.unwrap().coverage, 0.25);
        let sun = scene.lighting.light.intensity;
        assert!((sun - 4.0 * 0.55).abs() < 1e-5, "sun {sun}");
    }

    /// A background that is the scene's own picture gets no deck: the
    /// weather dims its light alone.
    #[test]
    fn a_scene_with_its_own_background_keeps_it() {
        let mut scene = RenderScene::new();
        scene.lighting.light.intensity = 4.0;
        let procedural = matches!(
            scene.lighting.environment.background,
            Background::Procedural { .. }
        );
        assert!(!procedural, "the default background is not the sky");
        look(WEATHER_RAIN, 1.0, 1.0).light_scene(&mut scene);
        assert!(scene.atmosphere.clouds.is_none());
        let sun = scene.lighting.light.intensity;
        assert!((sun - 4.0 * 0.45).abs() < 1e-5, "sun {sun}");
    }
}
