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
        }
    }
}

impl SurfaceWeather {
    /// The frame uniform's two rows (`frame.weather`, `frame.weather_look`).
    pub(crate) fn gpu(&self) -> [[f32; 4]; 2] {
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
        ]
    }
}

/// How bright a strike is `t` seconds after it: a sharp first stroke, a
/// dip, a second return stroke, then a fade (0 from 0.6 s).
pub fn lightning_flash(t: f32) -> f32 {
    let pulse = |c: f32, w: f32| (1.0 - ((t - c) / w).abs()).max(0.0);
    (pulse(0.03, 0.05)
        + 0.8 * pulse(0.17, 0.07)
        + 0.35 * (1.0 - t / 0.6).max(0.0) * f32::from(t > 0.05))
    .min(1.0)
        * f32::from(t < 0.6)
}

/// The bolt of strike number `strike`: a jagged line from (at.x, top, at.z)
/// down to `at` by midpoint displacement from a generator seeded by the
/// strike, so the same strike draws the same bolt. (point, width) pairs.
pub fn lightning_bolt(at: Vec3, top: f32, strike: u32) -> Vec<(Vec3, f32)> {
    // A xorshift stream seeded by the strike.
    let mut state = strike.wrapping_mul(0x9e37_79b9) ^ 0x4c49_4748 | 1;
    let mut draw = move |r: f32| {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        (state as f32 / u32::MAX as f32 * 2.0 - 1.0) * r
    };
    let mut pts = vec![Vec3::new(at.x, top, at.z), at];
    let mut jag = (top - at.y) * 0.18;
    for _ in 0..6 {
        let mut next = Vec::with_capacity(pts.len() * 2);
        for pair in pts.windows(2) {
            let mid = (pair[0] + pair[1]) * 0.5 + Vec3::new(draw(jag), 0.0, draw(jag));
            next.push(pair[0]);
            next.push(mid);
        }
        next.push(*pts.last().unwrap_or(&at));
        pts = next;
        jag *= 0.55;
    }
    let n = pts.len() as f32;
    pts.into_iter()
        .enumerate()
        .map(|(i, p)| (p, 0.35 + 0.3 * i as f32 / n))
        .collect()
}

/// A lightning strike `seconds` after it struck `at`, from a cloud base at
/// height `top`, added to `scene` (nothing from 0.6 s): the double flash
/// brightens the sky's light and turns the scene's directional light to
/// the bolt's channel while it outshines it, the clouds (when the scene
/// has them) glow from inside 150 m over the cloud base above the strike
/// ([`crate::Clouds::flash`]), a bright blue-white light hangs 100 m over
/// the strike, lighting the ground with a falloff from it, and for 0.35 s
/// the bolt of strike number `strike` ([`lightning_bolt`]) is drawn as an
/// additive strip that blooms.
pub fn add_lightning(
    scene: &mut crate::RenderScene,
    at: Vec3,
    top: f32,
    strike: u32,
    seconds: f32,
) {
    let flash = lightning_flash(seconds);
    if flash <= 0.0 {
        return;
    }
    scene.environment.intensity *= 1.0 + 6.0 * flash;
    // The bolt lights the land like a low sun for the flash's length, with
    // hard shadows away from it: the scene's directional light turns to the
    // channel's lower part (150 m up) while the flash outshines it.
    let channel = at + Vec3::Y * 150.0;
    let toward = (channel - scene.camera.position).normalize_or(Vec3::Y);
    let bolt_light = 2.0 * flash;
    // The cloud over the root glows from inside (the clouds' own flash).
    if let Some(c) = scene.clouds.as_mut() {
        c.flash = Vec3::new(at.x, top + 150.0, at.z).extend(40.0 * flash);
    }
    let own = scene.light.intensity * scene.light.color.max_element();
    if bolt_light > own {
        let k = own / bolt_light;
        scene.light.direction_to_light = toward
            .lerp(scene.light.direction_to_light, k)
            .normalize_or(toward);
        scene.light.color = Vec3::new(0.78, 0.85, 1.0).lerp(scene.light.color, k);
        scene.light.intensity = bolt_light + own * k;
    }
    scene.lights.push(crate::Light {
        kind: crate::LightKind::Point,
        position: at + Vec3::Y * 100.0,
        color: Vec3::new(0.75, 0.82, 1.0),
        intensity: 2.0e4 * flash,
        range: 600.0,
        cast_shadows: false,
    });
    if seconds < 0.35 {
        // Bright enough to bloom, not so bright the bloom and its
        // reflection burn a white column across the frame.
        let c = [0.8 + 2.0 * flash, 0.9 + 2.3 * flash, 1.4 + 3.2 * flash, 1.0];
        scene.ribbons.push(crate::Ribbon {
            points: lightning_bolt(at, top, strike)
                .into_iter()
                .map(|(position, width)| crate::RibbonPoint {
                    position,
                    width,
                    color: c,
                })
                .collect(),
            texture: None,
            uv: crate::RibbonUv::Stretch,
            blend: ParticleBlend::Additive,
        });
    }
}

/// How far above and below the camera the shelter map reaches, in metres
/// along the fall.
pub(crate) const SHELTER_REACH: f32 = 400.0;
/// The shelter map's cells per side of its square: the camera moving a
/// cell redraws it.
pub(crate) const SHELTER_CELLS: f32 = 8.0;
/// How wide the drip line's soft edge is, in metres.
pub(crate) const DRIP_EDGE: f32 = 0.6;

/// The weather occlusion map's view (the R4 shelter): an orthographic
/// projection looking along `w.fall` over the `w.shelter` square around
/// `camera` (frame coordinates), centred on the snapped cell of the
/// camera's world position (`camera + origin`), so it moves only when the
/// camera crosses a cell. `None` without shelter. Its depth is what the
/// shadow pass writes for a directional cascade (clip z), and a surface is
/// sheltered where something nearer the sky along the fall was drawn.
/// Returns the view projection and the cell.
pub(crate) fn shelter_view(
    w: &SurfaceWeather,
    camera: Vec3,
    origin: Vec3,
) -> Option<(glam::Mat4, [i64; 2])> {
    if w.shelter <= 0.0 {
        return None;
    }
    let side = w.shelter.clamp(4.0, 1024.0);
    let cell = side / SHELTER_CELLS;
    let world = camera + origin;
    let ix = (world.x / cell).floor() as i64;
    let iz = (world.z / cell).floor() as i64;
    let centre = Vec3::new((ix as f32 + 0.5) * cell, world.y, (iz as f32 + 0.5) * cell) - origin;
    let d = w.fall.normalize_or(Vec3::NEG_Y);
    let up = if d.y.abs() > 0.9 { Vec3::Z } else { Vec3::Y };
    let eye = centre - d * SHELTER_REACH;
    let view = glam::camera::rh::view::look_at_mat4(eye, centre, up);
    let h = side * 0.5;
    let proj =
        glam::camera::rh::proj::directx::orthographic(-h, h, -h, h, 0.0, 2.0 * SHELTER_REACH);
    Some((proj * view, [ix, iz]))
}

/// What the shelter map was drawn from: its cell, its fall and side, and
/// the casters inside its volume (their bounding spheres, hashed). The map
/// is drawn again only when this changes: the camera crosses a cell, the
/// weather turns, or geometry under the map moves, appears or goes (a
/// static shadow cache, when the shadows have one, is where this should
/// later live with theirs).
pub(crate) fn shelter_key(
    w: &SurfaceWeather,
    cell: [i64; 2],
    spheres: impl Iterator<Item = (Vec3, f32)>,
    terrains: usize,
) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut put = |x: u64| {
        h ^= x;
        h = h.wrapping_mul(0x0100_0000_01b3);
    };
    put(cell[0] as u64);
    put(cell[1] as u64);
    for v in [w.fall.x, w.fall.y, w.fall.z, w.shelter] {
        put(u64::from(v.to_bits()));
    }
    put(terrains as u64);
    for (c, r) in spheres {
        for v in [c.x, c.y, c.z, r] {
            put(u64::from(v.to_bits()));
        }
    }
    h
}

#[cfg(test)]
mod lightning_tests {
    use super::*;

    /// The flash: bright at once, a second stroke after the dip, dark from
    /// 0.6 s. The bolt runs from the cloud to the strike, the same for the
    /// same strike and different for the next.
    #[test]
    fn a_strike_flashes_twice_and_draws_its_own_bolt() {
        assert!(lightning_flash(0.03) > 0.9);
        assert!(lightning_flash(0.17) > lightning_flash(0.11));
        assert_eq!(lightning_flash(0.6), 0.0);
        let at = Vec3::new(10.0, 2.0, -40.0);
        let a = lightning_bolt(at, 602.0, 3);
        assert_eq!(a.first().unwrap().0, Vec3::new(10.0, 602.0, -40.0));
        assert_eq!(a.last().unwrap().0, at);
        assert_eq!(a.len(), 65);
        assert_eq!(a, lightning_bolt(at, 602.0, 3));
        assert_ne!(a, lightning_bolt(at, 602.0, 4));
    }

    /// The shelter view moves by whole cells only, looks along the fall,
    /// and maps a point under the camera into the map; the key changes
    /// with the cell and with a caster moving, not with a camera move
    /// inside its cell.
    #[test]
    fn the_shelter_map_moves_by_cells_and_keys_on_its_casters() {
        let w = SurfaceWeather::default();
        let (a, cell) = shelter_view(&w, Vec3::new(1.0, 2.0, 1.0), Vec3::ZERO).unwrap();
        let (b, same) = shelter_view(&w, Vec3::new(3.0, 2.0, 2.0), Vec3::ZERO).unwrap();
        assert_eq!(cell, same);
        assert_eq!(a, b);
        let (_, next) = shelter_view(&w, Vec3::new(9.0, 2.0, 1.0), Vec3::ZERO).unwrap();
        assert_ne!(cell, next);
        // A roof 3 m up is nearer the sky (smaller depth) than the ground.
        let depth = |p: Vec3| {
            let c = a * p.extend(1.0);
            c.z / c.w
        };
        assert!(depth(Vec3::new(1.0, 3.0, 1.0)) < depth(Vec3::new(1.0, 0.0, 1.0)));
        let c = a * Vec3::new(1.0, 0.0, 1.0).extend(1.0);
        assert!(c.x.abs() < 1.0 && c.y.abs() < 1.0 && (0.0..1.0).contains(&(c.z / c.w)));
        let caster = [(Vec3::new(0.0, 3.0, 0.0), 1.0)];
        let k = shelter_key(&w, cell, caster.into_iter(), 0);
        assert_eq!(k, shelter_key(&w, same, caster.into_iter(), 0));
        assert_ne!(k, shelter_key(&w, next, caster.into_iter(), 0));
        let moved = [(Vec3::new(0.5, 3.0, 0.0), 1.0)];
        assert_ne!(k, shelter_key(&w, cell, moved.into_iter(), 0));
        assert!(
            shelter_view(
                &SurfaceWeather { shelter: 0.0, ..w },
                Vec3::ZERO,
                Vec3::ZERO
            )
            .is_none()
        );
    }
}
