//! Weather presets (the E5 round): rain, snow, dust and mist as a particle
//! emitter, a fog and a dimming of the light, tuned by intensity, wind and
//! the size of the region around the camera they fill. The runtime places
//! the emitter over the camera each frame; the particles are simulated by
//! the renderer from the tick like every particle system.

use crate::{
    ColorCurve, Curve, EmitterShape, Fog, ParticleAlign, ParticleBlend, ParticleEmitter,
    ParticleNoise, ParticleRender, SimulationSpace,
};
use glam::Vec3;

/// Clear weather.
pub const WEATHER_NONE: u32 = 0;
/// Rain streaks falling with the wind, dimmed light, a grey distance fog.
pub const WEATHER_RAIN: u32 = 1;
/// Flakes drifting down on gusts, a pale fog.
pub const WEATHER_SNOW: u32 = 2;
/// Dust blowing along the wind near the ground, a warm thick fog.
pub const WEATHER_DUST: u32 = 3;
/// No particles, a dense low height fog.
pub const WEATHER_MIST: u32 = 4;

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
    /// A multiplier on the directional light (and half as much on the ambient).
    pub light: f32,
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
    let i = intensity.clamp(0.0, 2.0);
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
    let wind = if two_d {
        Vec3::new(wind.x, wind.y, 0.0)
    } else {
        wind
    };
    let tinted = |c: [f32; 4]| [c[0] * tint.x, c[1] * tint.y, c[2] * tint.z, c[3]];
    let ahead = if two_d { 0.0 } else { r * 0.6 };
    let dimmed = |amount: f32| 1.0 - amount * (i * dim.clamp(0.0, 1.0)).min(1.0);
    // A particle with drag d under acceleration g settles at g / d: the
    // fall and the wind are set as that terminal velocity.
    let drifting = |fall: f32, drag: f32| (wind + Vec3::new(0.0, -fall, 0.0)) * drag;
    match kind {
        WEATHER_RAIN => {
            let fall = 11.0;
            let height = r * 1.2;
            WeatherLook {
                emitter: Some(ParticleEmitter {
                    max_particles: (6000.0 * i).ceil().max(1.0) as u32,
                    // Born anywhere in the column, so the view fills at once.
                    shape: EmitterShape::Box {
                        half_extents: half(r, height * 0.5),
                    },
                    rate: (r * r * 14.0 * i).min(60_000.0),
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
            }
        }
        WEATHER_SNOW => {
            let fall = 1.1;
            let height = r * 1.2;
            WeatherLook {
                emitter: Some(ParticleEmitter {
                    max_particles: (16000.0 * i).ceil().max(1.0) as u32,
                    shape: EmitterShape::Box {
                        half_extents: half(r, height * 0.5),
                    },
                    rate: (r * r * 12.0 * i).min(20_000.0),
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
            }
        }
        WEATHER_DUST => WeatherLook {
            emitter: Some(ParticleEmitter {
                max_particles: (2500.0 * i).ceil().max(1.0) as u32,
                shape: EmitterShape::Box {
                    half_extents: half(r, if two_d { r * 0.6 } else { 1.5 }),
                },
                rate: (r * r * 0.8 * i).min(8000.0),
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
        },
        _ => WeatherLook {
            emitter: None,
            offset: Vec3::ZERO,
            ahead: 0.0,
            fog: None,
            light: 1.0,
        },
    }
}

impl WeatherLook {
    /// Where the emitter goes for a camera at `position` looking along
    /// `forward`.
    pub fn origin(&self, position: Vec3, forward: Vec3) -> Vec3 {
        let level = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
        position + self.offset + level * self.ahead
    }
}
