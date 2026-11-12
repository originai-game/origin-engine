//! The E5 round's rendering from the world (`og.atmosphere` and the
//! drawing of the physics that has a look): og.Fog, og.Sky, og.DayNight
//! and og.Weather into the scene's fog, sky, light and particles; og.Water
//! (with an og.WaterSurface's heights), og.Water2D over an og.WaterLine,
//! og.Fluid2D as metaballs (a group as one surface), og.Fluid3D as a
//! screen-space surface (a group's particles coloured by their mix), visible og.Rope and
//! og.Cloth; and the material shaders (`og.load.shader`, `og.render.shaderMaterial`,
//! `og.render.setShaderParam`). Like the rest of extraction it reads the
//! world (and the physics' last step) and never writes either.

use super::codec::Layout;
use super::engine::engine;
use super::extract::{Hierarchy, RenderState};
use super::fx::{kept_rows, resource, rows};
use super::parse;
use super::wire::{Color, FromJson, WindMotorKind, record};
use glam::{Mat4, Vec2, Vec3};
use og_core::IdMap;
use og_ecs::{Entity, World};
use og_render::{
    Background, Fog, MaterialHandle, MeshData, MeshHandle, Metaballs, ParticleSystem,
    ProceduralSky, Projection, Ribbon, RibbonPoint, RibbonUv, SkyLook, Sprite, Vertex, Water,
    Water2d, WaterCut, WaterHeights, WaterWakes,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::Arc;

/// The layouts of the records this module reads, and the physics whose
/// last step it draws.
#[derive(Default)]
pub(crate) struct AtmoLayouts {
    fog: Option<Layout>,
    sky: Option<Layout>,
    physical_sky: Option<Layout>,
    clouds: Option<Layout>,
    aurora: Option<Layout>,
    day_night: Option<Layout>,
    weather: Option<Layout>,
    weather_state: Option<Layout>,
    wind: Option<Layout>,
    wind_motor: Option<Layout>,
    water: Option<Layout>,
    water_2d: Option<Layout>,
    water_line: Option<Layout>,
    fluid: Option<Layout>,
    fluid3d: Option<Layout>,
    rope: Option<Layout>,
    cloth: Option<Layout>,
    soft: Option<Layout>,
    pub ragdoll: Option<Layout>,
    pub physics: Option<og_physics::PhysicsSystem>,
}

impl AtmoLayouts {
    pub fn from(
        get: impl Fn(&str) -> Option<Layout>,
        physics: Option<og_physics::PhysicsSystem>,
    ) -> AtmoLayouts {
        AtmoLayouts {
            fog: get("og.Fog"),
            sky: get("og.Sky"),
            physical_sky: get("og.PhysicalSky"),
            clouds: get("og.Clouds"),
            aurora: get("og.Aurora"),
            day_night: get("og.DayNight"),
            weather: get("og.Weather"),
            weather_state: get("og.WeatherState"),
            wind: get("og.Wind"),
            wind_motor: get("og.WindMotor"),
            water: get("og.Water"),
            water_2d: get("og.Water2D"),
            water_line: get("og.WaterLine"),
            fluid: get("og.Fluid2D"),
            fluid3d: get("og.Fluid3D"),
            rope: get("og.Rope"),
            cloth: get("og.Cloth"),
            soft: get("og.SoftBody"),
            ragdoll: get("og.Ragdoll"),
            physics,
        }
    }
}

/// What this module keeps between frames: the weather's emitter, the
/// cloths' meshes, the loaded shaders and the shader materials.
#[derive(Default)]
pub(crate) struct AtmoState {
    weather: Option<(Vec<u32>, og_render::EmitterHandle)>,
    cloths: IdMap<u64, MeshHandle>,
    cloth_material: Option<MaterialHandle>,
    softs: IdMap<u64, MeshHandle>,
    soft_material: Option<MaterialHandle>,
    /// `og.load.shader` handles by load index, and loads by path.
    pub shaders: Vec<og_render::ShaderHandle>,
    pub shader_paths: BTreeMap<String, usize>,
    /// Ragdoll rigs by entity (`og.physics.setRagdollRig`), replayed into
    /// a reloaded world's physics, and this frame's palette of each posed
    /// model by entity.
    pub rigs: IdMap<u64, super::physics_ext::KeptRig>,
    pub palettes: IdMap<u64, u32>,
}

/// The particle system id of the weather (no entity has it).
const WEATHER_ID: u64 = u64::MAX - 0x5745;

/// The DayNight clock at `tick` (hours, 0..24), from its row.
fn hours(d: &record::DayNight, tick: u64, dt: f32) -> f32 {
    let start = f64::from(d.start);
    if !d.enabled {
        return start.rem_euclid(24.0) as f32;
    }
    let speed = f64::from(d.speed);
    (start + tick as f64 * f64::from(dt) * speed).rem_euclid(24.0) as f32
}

/// An enabled og.DayNight at the world's tick: its row, the clock, the
/// look, and the directional light's intensity (the look's times
/// lightIntensity).
struct DayNightNow {
    d: record::DayNight,
    hours: f32,
    look: og_render::DayNightLook,
    intensity: f32,
}

/// What og.DayNight makes of the frame at the world's tick, or `None` when it
/// is absent or disabled. A pure function of the world: extraction draws
/// it, and the probe's `get` answers it ([`derive`]), so what is read is
/// what is drawn.
fn day_night_now(
    world: &World,
    layout: Option<&Layout>,
    physical: Option<&Layout>,
) -> Option<DayNightNow> {
    let d = record::DayNight::from_json(&*resource(world, layout)?).ok()?;
    if !d.enabled {
        return None;
    }
    let time = world.time();
    let (tilt, azimuth) = (d.tilt, d.azimuth);
    let now = hours(&d, time.tick, time.dt);
    let air = physical_air(world, physical);
    let look = og_render::day_night(now, tilt, azimuth);
    let look = match &air {
        Some(a) => look.with_atmosphere(a),
        None => look,
    };
    let intensity = look.light_intensity * d.light_intensity.max(0.0);
    Some(DayNightNow {
        d,
        hours: now,
        look,
        intensity,
    })
}

/// An enabled og.PhysicalSky as the renderer's atmosphere.
fn physical_air(world: &World, layout: Option<&Layout>) -> Option<og_render::PhysicalAtmosphere> {
    let p = record::PhysicalSky::from_json(&*resource(world, layout)?).ok()?;
    if !p.enabled {
        return None;
    }
    let earth = og_render::PhysicalAtmosphere::EARTH;
    Some(og_render::PhysicalAtmosphere {
        rayleigh: earth.rayleigh * p.air_density.max(0.0),
        mie: earth.mie * p.haze.max(0.0),
        mie_absorption: earth.mie_absorption * p.haze.max(0.0),
        ozone: earth.ozone * p.ozone.max(0.0),
        ground_albedo: rgb(&p.ground_albedo),
        altitude: p.altitude.max(0.0),
        night: rgb(&p.night),
        ..earth
    })
}

#[cfg(not(feature = "publish"))]
fn xyz(v: Vec3) -> Value {
    json!([v.x, v.y, v.z])
}

/// What the probe's `get` answers for records og.DayNight drives (the
/// `og_probe::DeriveFn` the runtime installs): og.DayNight gains the clock
/// and the light it makes (`hours`, `toSun`, `toLight`, `lightColor`,
/// `lightIntensity`); the first og.DirectionalLight, the one it drives, is
/// answered with the colour and intensity it is drawn with, `toLight` (the
/// unit vector towards the sun or the moon that replaces the entity's -Z)
/// and `drivenBy`. Reads only; the world is never written. A publish
/// build has no `get`, so no derive.
#[cfg(not(feature = "publish"))]
pub(crate) fn derive(
    world: &World,
    layouts: &BTreeMap<String, Layout>,
    entity: Option<Entity>,
    name: &str,
    fields: &mut serde_json::Map<String, Value>,
) {
    let driven = match (name, entity) {
        ("og.DayNight", None) => false,
        ("og.DirectionalLight", Some(e)) => {
            let first = rows(world, layouts.get("og.DirectionalLight"))
                .into_iter()
                .next()
                .map(|(bits, _)| bits);
            if first != Some(e.to_bits()) {
                return;
            }
            true
        }
        _ => return,
    };
    let Some(now) = day_night_now(
        world,
        layouts.get("og.DayNight"),
        layouts.get("og.PhysicalSky"),
    ) else {
        return;
    };
    let c = now.look.light_color;
    if driven {
        // The stored colour's lanes (a fourth, if any, kept).
        let mut color = fields.get("color").cloned().unwrap_or(json!([1, 1, 1]));
        if let Some(lanes) = color.as_array_mut() {
            for (i, v) in [c.x, c.y, c.z].into_iter().enumerate() {
                if let Some(slot) = lanes.get_mut(i) {
                    *slot = json!(v);
                }
            }
        }
        fields.insert("color".into(), color);
        fields.insert("intensity".into(), json!(now.intensity));
        fields.insert("toLight".into(), xyz(now.look.light_direction));
        fields.insert("drivenBy".into(), json!("og.DayNight"));
    } else {
        fields.insert("hours".into(), json!(now.hours));
        fields.insert("toSun".into(), xyz(now.look.sun_direction));
        fields.insert("toLight".into(), xyz(now.look.light_direction));
        fields.insert("lightColor".into(), xyz(c));
        fields.insert("lightIntensity".into(), json!(now.intensity));
    }
}

/// A river's path handle and its width's and speed's bits.
type RiverKey = (u64, u32, u32);

/// The water on one engine: its rivers by [`RiverKey`], the newest last.
#[derive(Default)]
pub(crate) struct ModuleState {
    rivers: std::cell::RefCell<Vec<(RiverKey, Arc<og_core::river::River>)>>,
}

/// An og.Water's river (og.Water.river: its path sampled every metre or so,
/// riverWidth and riverSpeed), the one record the tick drifts floaters with
/// and the renderer draws: made once per path, width and speed (the same
/// `Arc` frame after frame, so the renderer keeps its ribbon). None without
/// a live path.
pub(crate) fn river_of(engine: u32, v: &Value) -> Option<Arc<og_core::river::River>> {
    let eng = super::engine::engine(engine).ok()?;
    let rivers = &eng.modules.atmosphere.rivers;
    let w = record::Water::from_json(v).ok()?;
    let handle = Some(w.river).filter(|h| *h != 0)?;
    let (width, speed) = (w.river_width.max(0.0), w.river_speed.max(0.0));
    let key = (handle, width.to_bits(), speed.to_bits());
    if let Some(r) = rivers
        .borrow()
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, r)| r.clone())
    {
        return Some(r);
    }
    let path = super::timeline::path_of(engine, handle)?;
    // A point every metre, at least the path's own segments, at most 2048.
    let n = (path.length().ceil() as usize)
        .max(path.segments() + 1)
        .clamp(2, 2048);
    let points: Vec<Vec3> = (0..n)
        .map(|i| path.sample(i as f32 / (n - 1) as f32).position)
        .collect();
    let river = Arc::new(og_core::river::River::new(&points, width, speed)?);
    let mut c = rivers.borrow_mut();
    if c.len() >= 64 {
        c.remove(0);
    }
    c.push((key, river.clone()));
    Some(river)
}

/// og.Fog, og.Sky, og.DayNight, og.Weather, og.Wind, og.Water, og.Water2D,
/// og.Fluid2D, og.Fluid3D, og.Rope and og.Cloth into `rs.scene`. Returns the keys of
/// the instances it added (cloth), for per-object motion.
// Rendering only: drawn lines and fluid volumes.
#[allow(clippy::disallowed_methods)]
pub(crate) fn extract(world: &World, rs: &mut RenderState, h: &mut Hierarchy) -> Vec<(u64, u32)> {
    let mut keys = Vec::new();
    let RenderState {
        scene, draw, atmo, ..
    } = rs;
    let layouts = &draw.atmo;
    scene.atmosphere.fog = None;
    scene.atmosphere.volumetric_fog = None;
    // No twilight adaptation unless the day-night cycle sets one below.
    scene.atmosphere.adaptation = 1.0;
    scene.atmosphere.wind = resource(world, layouts.wind.as_ref()).and_then(|w| {
        // The hand read's direction default (zero), not the IDL's +X.
        let base = record::Wind {
            direction: Vec3::ZERO,
            ..Default::default()
        };
        let w = record::Wind::from_json_over(&w, base).ok()?;
        (w.strength > 0.0).then_some(og_render::Wind {
            strength: w.strength,
            direction: w.direction,
            speed: w.speed,
            gust: w.gust,
            turbulence: w.turbulence,
            gust_size: w.gust_size,
        })
    });
    // og.WindMotor: local pushes on the simulated wind field, placed by
    // their entities (wind_field.rs keeps the nearest).
    scene.atmosphere.wind_motors.clear();
    for (e, v) in kept_rows(world, layouts.wind_motor.as_ref()) {
        let Ok(v) = record::WindMotor::from_json(&v) else {
            continue;
        };
        let m = h.world_of(e);
        let kind = match v.kind {
            WindMotorKind::Directional => og_render::WindMotorKind::Directional,
            WindMotorKind::Vortex => og_render::WindMotorKind::Vortex,
            WindMotorKind::Omni => og_render::WindMotorKind::Omni,
        };
        scene.atmosphere.wind_motors.push(og_render::WindMotor {
            position: m.transform_point3(Vec3::ZERO),
            direction: m.transform_vector3(Vec3::NEG_Z),
            radius: v.radius,
            strength: v.strength,
            kind,
        });
    }
    scene.world.water.clear();
    scene.sprite.water_2d.clear();
    scene.sprite.metaballs.clear();
    scene.world.fluids_3d.clear();
    let time = world.time();
    let tick = time.tick;
    let orthographic = matches!(
        scene.frame.camera.projection,
        Projection::Orthographic { .. }
    );

    // The physical sky: alone, towards the first directional light.
    let air = physical_air(world, layouts.physical_sky.as_ref());
    if let Some(air) = air {
        let dir = scene
            .lighting
            .light
            .direction_to_light
            .normalize_or(Vec3::Y);
        let sun = Vec3::new(1.0, 0.97, 0.93);
        let look = SkyLook {
            physical: Some(air),
            sun_size: 0.8,
            ..SkyLook::DEFAULT
        };
        scene.lighting.environment.sky = ProceduralSky {
            sun_direction: dir,
            sun_color: sun,
            look: Some(look),
        };
        scene.lighting.environment.background = Background::Procedural {
            sun_direction: dir,
            sun_color: sun,
        };
    }

    // The sky: a chosen look, drawn per pixel and lit by.
    if let Some(s) = resource(world, layouts.sky.as_ref())
        // The hand read's sunDirection default (zero, so +Y below), not
        // the IDL's (0.45, 0.8, 0.35).
        && let Ok(s) = record::Sky::from_json_over(
            &s,
            record::Sky {
                sun_direction: Vec3::ZERO,
                ..Default::default()
            },
        )
        && s.enabled
    {
        let look = SkyLook {
            zenith: rgb(&s.zenith),
            horizon: rgb(&s.horizon),
            ground: rgb(&s.ground),
            sun_size: s.sun_size,
            sun_intensity: s.sun_intensity,
            glow: s.glow,
            glow_power: s.glow_power,
            stars: s.stars,
            brightness: s.brightness,
            moon_size: s.moon_size,
            moon_phase: s.moon_phase,
            moon_intensity: s.moon_intensity,
            moon_color: rgb(&s.moon_color),
            moon_direction: s.moon_direction,
            physical: air,
        };
        let dir = s.sun_direction.normalize_or(Vec3::Y);
        let sun = rgb(&s.sun_color);
        scene.lighting.environment.sky = ProceduralSky {
            sun_direction: dir,
            sun_color: sun,
            look: Some(look),
        };
        scene.lighting.environment.background = Background::Procedural {
            sun_direction: dir,
            sun_color: sun,
        };
    }

    // The day-night cycle: light, sky and ambient from the clock.
    let mut fog_tint = None;
    if let Some(DayNightNow {
        d,
        hours: now,
        look,
        intensity,
    }) = day_night_now(
        world,
        layouts.day_night.as_ref(),
        layouts.physical_sky.as_ref(),
    ) {
        let (tilt, azimuth) = (d.tilt, d.azimuth);
        // The image-based light steps every `step` game minutes.
        let step = d.step.clamp(1.0, 120.0) / 60.0;
        let mut stepped = og_render::day_night((now / step).floor() * step, tilt, azimuth);
        if let Some(a) = &air {
            stepped = stepped.with_atmosphere(a);
        }
        // The moon opposite the sun drawn this frame (moon_direction zero).
        stepped.sky.moon_size = d.moon_size;
        stepped.sky.moon_phase = d.moon_phase;
        scene.lighting.light.direction_to_light = look.light_direction;
        scene.lighting.light.color = look.light_color;
        scene.lighting.light.intensity = intensity;
        scene.lighting.environment.intensity = look.ambient;
        scene.atmosphere.adaptation = look.adaptation;
        scene.lighting.environment.sky = ProceduralSky {
            sun_direction: stepped.sun_direction,
            sun_color: stepped.sun_color,
            look: Some(stepped.sky),
        };
        scene.lighting.environment.background = Background::Procedural {
            sun_direction: look.sun_direction,
            sun_color: look.sun_color,
        };
        if d.tint_fog {
            fog_tint = Some((look.fog_color, look.fog_sun));
        }
    }

    // Clouds over the procedural sky (they bring the built-in one when no
    // sky resource drew it), dimming the light by their cover.
    scene.atmosphere.clouds = resource(world, layouts.clouds.as_ref())
        // The hand read's wind default (zero), not the IDL's (12, 0.4, 4).
        .and_then(|c| {
            let base = record::Clouds {
                wind: Vec3::ZERO,
                ..Default::default()
            };
            record::Clouds::from_json_over(&c, base).ok()
        })
        .filter(|c| c.enabled)
        .map(|c| og_render::Clouds {
            coverage: c.coverage,
            density: c.density,
            base: c.base,
            thickness: c.thickness,
            scale: c.scale,
            wind: c.wind,
            color: rgb(&c.color),
            detail: c.detail,
            layer: c.layer,
            layer_altitude: c.layer_altitude,
            steps: c.steps,
            // A strike's flash, set by lightning_and_surfaces.
            flash: glam::Vec4::ZERO,
        });
    // The aurora, in the same procedural sky.
    scene.atmosphere.aurora = resource(world, layouts.aurora.as_ref())
        .and_then(|a| record::Aurora::from_json(&a).ok())
        .filter(|a| a.enabled)
        .map(|a| og_render::Aurora {
            intensity: a.intensity,
            activity: a.activity,
            curtains: a.curtains,
            direction: a.direction,
            distance: a.distance,
            spread: a.spread,
            energy: a.energy,
            speed: a.speed,
            seed: a.seed,
            green: rgb(&a.green_color),
            red: rgb(&a.red_color),
            nitrogen: rgb(&a.nitrogen_color),
        });
    if scene.atmosphere.aurora.is_some()
        && scene.atmosphere.clouds.is_none()
        && !matches!(
            scene.lighting.environment.background,
            Background::Procedural { .. }
        )
    {
        let dir = scene
            .lighting
            .light
            .direction_to_light
            .normalize_or(Vec3::Y);
        scene.lighting.environment.background = Background::Procedural {
            sun_direction: dir,
            sun_color: scene.lighting.environment.sky.sun_color,
        };
    }
    if scene.atmosphere.clouds.is_some()
        && !matches!(
            scene.lighting.environment.background,
            Background::Procedural { .. }
        )
    {
        let dir = scene
            .lighting
            .light
            .direction_to_light
            .normalize_or(Vec3::Y);
        scene.lighting.environment.background = Background::Procedural {
            sun_direction: dir,
            sun_color: scene.lighting.environment.sky.sun_color,
        };
    }

    // Fog, declared or from the weather.
    if let Some(v) = resource(world, layouts.fog.as_ref())
        && let Ok(v) = record::Fog::from_json(&v)
        && v.enabled
    {
        let mut fog = Fog {
            color: rgb(&v.color),
            density: v.density,
            start: v.start,
            end: v.end,
            height_density: v.height_density,
            height: v.height,
            falloff: v.falloff,
            sun_color: rgb(&v.sun_color),
            sun_power: v.sun_power,
            max_opacity: v.max_opacity,
            sky: v.sky,
        };
        if let Some((t, sun)) = fog_tint {
            fog.color = fog.color.lerp(t, 0.75);
            if let Some(sun) = sun {
                fog.sun_color = sun;
            }
        }
        // Fog.volumetric: the same medium lit (og-render's volumetric.rs),
        // which replaces the plain fog where the renderer draws it.
        if v.volumetric {
            scene.atmosphere.volumetric_fog = Some(og_render::VolumetricFog {
                color: fog.color,
                density: fog.density,
                height: fog.height,
                height_density: fog.height_density,
                falloff: fog.falloff,
                scattering: v.scattering,
                anisotropy: v.anisotropy,
                distance: v.volumetric_distance,
            });
        }
        scene.atmosphere.fog = Some(fog);
    }
    if let Some(c) = scene.atmosphere.clouds {
        c.light_scene(scene);
    }
    if let Some(w) = resource(world, layouts.weather.as_ref())
        && let Ok(w) = record::Weather::from_json(&w)
    {
        let kind = w.kind;
        let layer = orthographic.then_some(w.layer);
        let wind = w.wind;
        let look = og_render::weather(
            kind,
            w.intensity,
            wind,
            w.radius,
            rgb(&w.tint),
            w.dim,
            layer,
        );
        if scene.atmosphere.fog.is_none()
            && let Some(mut fog) = look.fog
        {
            if let Some((t, sun)) = fog_tint {
                fog.color = fog.color.lerp(t, 0.5);
                if let Some(sun) = sun {
                    fog.sun_color = sun;
                }
            }
            scene.atmosphere.fog = Some(fog);
        }
        scene.lighting.light.cover(look.light);
        scene.lighting.environment.intensity *= 0.5 + 0.5 * look.light;
        if let Some(emitter) = &look.emitter {
            // The emitter is remade only when its inputs change.
            let key: Vec<u32> = [
                kind as f32,
                w.intensity,
                wind.x,
                wind.y,
                wind.z,
                w.radius,
                layer.map_or(f32::NAN, |l| l as f32),
            ]
            .iter()
            .map(|x| x.to_bits())
            .chain([w.tint.r.to_bits(), w.tint.g.to_bits(), w.tint.b.to_bits()])
            .collect();
            let handle = match atmo.weather.as_ref().map(|(k, h)| (k == &key, *h)) {
                Some((true, handle)) => handle,
                Some((false, handle)) => {
                    scene.set_emitter(handle, emitter.clone());
                    atmo.weather = Some((key, handle));
                    handle
                }
                None => {
                    let handle = scene.add_emitter(emitter.clone());
                    atmo.weather = Some((key, handle));
                    handle
                }
            };
            let cam = &scene.frame.camera;
            let at = look.origin(cam.position, cam.target - cam.position);
            let at = if orthographic {
                Vec3::new(cam.position.x, cam.position.y + look.offset.y, 0.0)
            } else {
                at
            };
            scene.effects.particles.push(ParticleSystem::new(
                WEATHER_ID,
                handle,
                Mat4::from_translation(at),
                1,
                tick,
            ));
        }
    }

    lightning_and_surfaces(world, scene, layouts, tick);

    // Ragdolls blended into their models' palettes.
    super::physics_ext::draw_ragdolls(world, rs, &mut |e| h.world_of(e));
    let engine = rs.engine;
    let RenderState {
        scene,
        draw,
        atmo,
        materials,
        geometries,
        textures,
        terrain,
        ..
    } = rs;
    let layouts = &draw.atmo;
    let physics = layouts.physics.as_ref();
    // The terrains a sea's seabed is read from (none in the 2D profile).
    #[cfg(not(feature = "profile2d"))]
    let terrain_layout = draw.terrain.as_ref();
    #[cfg(feature = "profile2d")]
    let terrain_layout = None;
    let texture = |h: u64| super::assets::index_of(h as f64).and_then(|i| textures.get(i).copied());
    // 3D water, with its wave heights when the entity has them.
    for (e, v) in kept_rows(world, layouts.water.as_ref()) {
        let Ok(row) = record::Water::from_json(&v) else {
            continue;
        };
        let entity = Entity::from_bits(e);
        let field = physics.and_then(|p| p.water_heights(entity));
        let wakes = physics.and_then(|p| p.water_wakes(entity)).map(|w| {
            Arc::new(WaterWakes {
                epoch: w.epoch,
                wave_speed: w.wave_speed,
                damping: w.damping,
                splash: w.splash,
                dt: w.dt,
                ticks: w
                    .ticks
                    .into_iter()
                    .map(|(tick, cuts)| {
                        let cuts = cuts
                            .into_iter()
                            .map(|c| WaterCut {
                                entity: c.entity,
                                min: c.min,
                                max: c.max,
                                centre: c.centre,
                                half: c.half,
                                axis: c.axis,
                                depth: c.depth,
                                speed: c.speed,
                            })
                            .collect();
                        (tick, cuts)
                    })
                    .collect(),
            })
        });
        let m = h.world_of(e);
        // og.Water.geometry: drawn over that mesh, in the entity's frame (scale included).
        let raw = row.geometry;
        let mesh = (raw != 0)
            .then(|| geometries.get((raw & 0xffff_ffff) as usize).copied())
            .flatten();
        let transform = if mesh.is_some() {
            m
        } else if field.is_some() {
            // The wave field is centred on the entity and unrotated.
            Mat4::from_translation(m.transform_point3(Vec3::ZERO))
        } else {
            let (_, r, t) = m.to_scale_rotation_translation();
            Mat4::from_rotation_translation(r, t)
        };
        scene.world.water.push(Water {
            transform,
            size: row.size,
            segments: row.segments,
            shallow: rgb(&row.shallow),
            deep: rgb(&row.deep),
            depth_fade: row.depth_fade,
            absorption: rgb(&row.absorption),
            scattering: rgb(&row.scattering),
            opacity: row.opacity,
            foam: rgb(&row.foam),
            foam_width: row.foam_width,
            wave_scale: row.wave_scale,
            wave_speed: row.wave_speed,
            wave_strength: row.wave_strength,
            refraction: row.refraction,
            reflectivity: row.reflectivity,
            heights: field.map(|w| WaterHeights {
                resolution: w.resolution,
                size: w.size,
                heights: Arc::from(w.heights),
            }),
            wakes,
            planar_reflection: row.planar_reflection,
            shadows: row.shadows,
            f0: row.f0,
            reflection_bias: row.reflection_bias,
            underwater_color: rgb(&row.underwater_color),
            underwater_density: row.underwater_density,
            caustics: row.caustics,
            caustics_scale: row.caustics_scale,
            mesh,
            normal_map: texture(row.normal_map),
            normal_scale: row.normal_scale.max(0.0),
            flow: row.flow,
            glint: row.glint.max(0.0),
            flow_map: texture(row.flow_map),
            wind: row.wind,
            swell: row.swell,
            choppiness: row.choppiness,
            foam_streaks: row.foam_streaks.max(0.0),
            spray: row.spray.max(0.0),
            dynamic_waves: row.dynamic_waves.max(0.0),
            meniscus: row.meniscus,
            endless: row.endless,
            shore: seabed(
                world,
                terrain,
                terrain_layout,
                entity,
                &v,
                &row,
                mesh.is_some(),
            ),
            river: river_of(engine, &v),
        });
    }
    // 2D water over its spring line.
    let lines: IdMap<u64, record::WaterLine> = IdMap::from_iter(
        kept_rows(world, layouts.water_line.as_ref())
            .into_iter()
            .filter_map(|(e, v)| Some((e, record::WaterLine::from_json(&v).ok()?))),
    );
    for (e, v) in kept_rows(world, layouts.water_2d.as_ref()) {
        let (Some(line), Some(p)) = (lines.get(&e), physics) else {
            continue;
        };
        let Ok(v) = record::Water2D::from_json(&v) else {
            continue;
        };
        let at = h.world_of(e).transform_point3(Vec3::ZERO);
        let width = line.width;
        let surface = p
            .water_line(Entity::from_bits(e))
            .unwrap_or_else(|| vec![0.0; line.columns.max(2) as usize]);
        scene.sprite.water_2d.push(Water2d {
            origin: Vec2::new(at.x - width * 0.5, at.y),
            width,
            depth: line.depth,
            surface,
            top: rgb(&v.top).extend(v.top_opacity),
            bottom: rgb(&v.bottom).extend(v.bottom_opacity),
            lip: rgb(&v.lip).extend(0.9),
            lip_width: v.lip_width,
            layer: v.layer,
            z: at.z,
        });
    }
    let Some(physics) = physics else {
        return keys;
    };
    // Fluids as metaballs.
    for (e, v) in kept_rows(world, layouts.fluid.as_ref()) {
        let Ok(v) = record::Fluid2D::from_json(&v) else {
            continue;
        };
        if !v.visible {
            continue;
        }
        let Some(particles) = physics.fluid_particles(Entity::from_bits(e)) else {
            continue;
        };
        // A fluid sharing its group is drawn with the group as one surface,
        // its particles by their mix (og-physics' fluid_mix: its lane).
        let (group, lane, mix) = match physics.fluid_mix(Entity::from_bits(e)) {
            Some((lane, mix)) => (v.group, lane, Arc::from(mix)),
            None => (0, 0, Arc::from(Vec::new())),
        };
        scene.sprite.metaballs.push(Metaballs {
            particles: Arc::from(particles),
            radius: v.radius * v.blob_scale,
            threshold: v.threshold,
            // Fluid2D.color is a Vector3 (x, y, z are r, g, b).
            color: v.color.extend(v.opacity),
            layer: v.layer,
            z: h.world_of(e).transform_point3(Vec3::ZERO).z,
            group,
            lane,
            mix,
            ..Metaballs::default()
        });
    }
    // 3D fluids as a screen-space surface. A fluid sharing a group colours
    // its particles by their mix over the group's fluids' colours (lanes as
    // og-physics' fluid3d_mix gives them).
    // Fluid3D.color is a Vector3 (x, y, z are r, g, b).
    let fluids3: Vec<(u64, record::Fluid3D)> = kept_rows(world, layouts.fluid3d.as_ref())
        .into_iter()
        .filter_map(|(e, v)| {
            // The hand read's size default (zero), not the IDL's (2, 1, 2).
            let base = record::Fluid3D {
                size: Vec3::ZERO,
                ..Default::default()
            };
            Some((e, record::Fluid3D::from_json_over(&v, base).ok()?))
        })
        .collect();
    let mut lanes: BTreeMap<(u32, u32), Vec3> = BTreeMap::new();
    let mut mixes = BTreeMap::new();
    for (e, v) in &fluids3 {
        if let Some((lane, mix)) = physics.fluid3d_mix(Entity::from_bits(*e)) {
            lanes.insert((v.group, lane), v.color);
            mixes.insert(*e, (v.group, mix));
        }
    }
    for (e, v) in &fluids3 {
        if !v.visible {
            continue;
        }
        let Some(particles) = physics.fluid3d_particles(Entity::from_bits(*e)) else {
            continue;
        };
        let radius = v.radius;
        // SIMULATION_VISUAL: the renderer's particles (the tick has none),
        // spawned and moved on the GPU from this.
        if visual(v.simulation) {
            let at = h.world_of(*e).transform_point3(Vec3::ZERO);
            let size = v.size;
            let count = v.count;
            let fluid = og_render::VisualFluid {
                key: *e,
                count,
                centre: at,
                size,
                bounds: v.bounds,
                cell: radius,
                max_count: v.max_count,
                // The solver's own are water at the component's defaults.
                stiffness: v.stiffness / 0.02,
                viscosity: v.viscosity / 0.4,
                gravity: physics.gravity().unwrap_or(Vec3::new(0.0, -9.81, 0.0)) * v.gravity_scale,
                substeps: v.substeps,
                seed: v.seed,
                emit_rate: v.emit_rate,
                emit_at: at + v.emit_offset,
                emit_radius: v.emit_radius,
                emit_velocity: v.emit_velocity,
                sink: {
                    let half = v.sink_size * 0.5;
                    let c = at + v.sink_offset;
                    half.cmpgt(Vec3::ZERO).all().then_some((c - half, c + half))
                },
            };
            // Each particle stands for the spawn box's volume over the
            // count, and is drawn at least as wide as that spacing.
            let volume = (size.x * size.y * size.z).max(1e-6) / count.max(1) as f32;
            scene.world.fluids_3d.push(og_render::Fluid3d {
                particles: Arc::from(Vec::new()),
                colors: None,
                color: v.color,
                opacity: v.opacity,
                radius: (radius * v.draw_scale).max(0.9 * volume.cbrt()),
                volume,
                reflectivity: v.reflectivity,
                refraction: v.refraction,
                smoothing: v.smoothing,
                visual: Some(Arc::new(fluid)),
            });
            continue;
        }
        let colors = mixes
            .get(e)
            .filter(|(_, mix)| mix.len() == particles.len())
            .map(|(group, mix)| {
                mix.iter()
                    .map(|m| {
                        (0..4u32)
                            .filter_map(|k| Some(*lanes.get(&(*group, k))? * m[k as usize]))
                            .sum::<Vec3>()
                    })
                    .collect::<Arc<[Vec3]>>()
            });
        scene.world.fluids_3d.push(og_render::Fluid3d {
            particles: Arc::from(particles),
            colors,
            color: v.color,
            opacity: v.opacity,
            radius: radius * v.draw_scale,
            // The share of the volume each particle stands for: at rest
            // they sit about half the interaction radius apart.
            volume: (radius * 0.5).powi(3),
            reflectivity: v.reflectivity,
            refraction: v.refraction,
            smoothing: v.smoothing,
            visual: None,
        });
    }
    // Ropes: a strip through their points in 3D, a sprite per segment in 2D.
    for (e, v) in kept_rows(world, layouts.rope.as_ref()) {
        // The hand read's colour default (black), not the IDL's brown.
        let base = record::Rope {
            color: Vec3::ZERO,
            ..Default::default()
        };
        let Ok(v) = record::Rope::from_json_over(&v, base) else {
            continue;
        };
        if !v.visible {
            continue;
        }
        let Some(points) = physics.rope_points(Entity::from_bits(e)) else {
            continue;
        };
        // Rope.color is a Vector3 (x, y, z are r, g, b).
        let c = v.color;
        let width = v.width;
        if orthographic {
            for pair in points.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                let d = b - a;
                scene.sprite.sprites.push(Sprite {
                    position: ((a + b) * 0.5).truncate().extend(a.z),
                    size: Vec2::new(d.truncate().length() + width, width),
                    rotation: d.y.atan2(d.x),
                    color: [c.x, c.y, c.z, 1.0],
                    ..Default::default()
                });
            }
        } else {
            scene.effects.ribbons.push(Ribbon {
                points: points
                    .iter()
                    .map(|&position| RibbonPoint {
                        position,
                        width: width * 1.6,
                        color: [c.x, c.y, c.z, 1.0],
                    })
                    .collect(),
                texture: None,
                uv: RibbonUv::Stretch,
                blend: og_render::ParticleBlend::Alpha,
                effect: None,
            });
        }
    }
    // og.Physics.gravity, for the bodies the renderer simulates.
    let gravity = physics.gravity().unwrap_or(Vec3::new(0.0, -9.81, 0.0));
    // Cloth: a double-sided mesh of its points, replaced every frame.
    for (e, v) in kept_rows(world, layouts.cloth.as_ref()) {
        let Ok(row) = record::Cloth::from_json(&v) else {
            continue;
        };
        if !row.visible {
            continue;
        }
        let Some(c) = physics.cloth_points(Entity::from_bits(e)) else {
            continue;
        };
        let mesh = cloth_mesh(c.columns, c.rows, &c.points);
        let handle = match atmo.cloths.get(&e) {
            Some(&handle) => handle,
            None => {
                let handle = scene.add_mesh(MeshData::default());
                atmo.cloths.insert(e, handle);
                handle
            }
        };
        scene.set_simulation_surface(
            handle,
            mesh,
            og_render::SimulationSurface::Cloth {
                columns: c.columns,
                rows: c.rows,
            },
        );
        if visual(row.simulation) {
            let body = og_render::VisualBody {
                pinned: c.pinned.into(),
                substeps: small_steps(row.substeps, row.iterations),
                damping: row.damping,
                gravity: gravity * row.gravity_scale,
                wind: row.wind,
                gust: row.gust,
                radius: row.radius,
                friction: row.friction,
                thickness: row.thickness,
                stiffness: 1.0,
                bending: CLOTH_BENDING,
                plane: c.flat.then(|| c.points.first().map_or(0.0, |p| p.z)),
                ..Default::default()
            };
            scene.set_visual_body(handle, Some(Arc::new(body)));
        }
        let material = super::assets::index_of(row.material as f64)
            .and_then(|i| materials.get(i).copied())
            .unwrap_or_else(|| {
                *atmo.cloth_material.get_or_insert_with(|| {
                    scene.add_standard_material(og_render::StandardMaterial {
                        color: Vec3::new(0.72, 0.16, 0.14),
                        roughness: 0.8,
                        side: 2,
                        ..Default::default()
                    })
                })
            });
        scene.add_instance(handle, material, Mat4::IDENTITY);
        keys.push((e, u32::MAX));
    }
    // Soft volumes: their surface, smooth-shaded, replaced every frame.
    for (e, v) in kept_rows(world, layouts.soft.as_ref()) {
        let Ok(row) = record::SoftBody::from_json(&v) else {
            continue;
        };
        if !row.visible {
            continue;
        }
        let Some(s) = physics.soft_body_points(Entity::from_bits(e)) else {
            continue;
        };
        let mesh = soft_mesh(&s.points, &s.indices);
        let handle = match atmo.softs.get(&e) {
            Some(&handle) => handle,
            None => {
                let handle = scene.add_mesh(MeshData::default());
                atmo.softs.insert(e, handle);
                handle
            }
        };
        scene.set_simulation_surface(handle, mesh, og_render::SimulationSurface::SoftBody);
        if visual(row.simulation) {
            let body = og_render::VisualBody {
                pinned: s.pinned.into(),
                substeps: small_steps(row.substeps, row.iterations),
                damping: row.damping,
                gravity: gravity * row.gravity_scale,
                radius: row.thickness,
                friction: row.friction,
                stiffness: row.stiffness,
                pressure: row.pressure,
                volume_stiffness: if row.pressure > 0.0 {
                    row.volume_stiffness
                } else {
                    0.0
                },
                plane: s.flat.then(|| s.points.first().map_or(0.0, |p| p.z)),
                ..Default::default()
            };
            scene.set_visual_body(handle, Some(Arc::new(body)));
        }
        let material = super::assets::index_of(row.material as f64)
            .and_then(|i| materials.get(i).copied())
            .unwrap_or_else(|| {
                *atmo.soft_material.get_or_insert_with(|| {
                    scene.add_standard_material(og_render::StandardMaterial {
                        color: Vec3::new(0.25, 0.75, 0.35),
                        roughness: 0.35,
                        side: 2,
                        ..Default::default()
                    })
                })
            });
        scene.add_instance(handle, material, Mat4::IDENTITY);
        keys.push((e, u32::MAX));
    }
    keys
}

/// The bending of a cloth the renderer simulates (og.Cloth has no field
/// for it; the tick's solve has no bending): enough to round its folds.
const CLOTH_BENDING: f32 = 0.2;

/// Whether a Cloth, SoftBody, Garment or Fluid3D `simulation` is
/// SIMULATION_VISUAL.
pub(crate) fn visual(simulation: u32) -> bool {
    simulation == og_physics::SIMULATION_VISUAL
}

/// The GPU solver's small steps per tick for a row's `substeps` and
/// `iterations`: twice their product, so its one pass a step converges as
/// the tick's iterations would (Macklin et al. 2019).
pub(crate) fn small_steps(substeps: u32, iterations: u32) -> u32 {
    let s = substeps.clamp(1, 16);
    let i = iterations.clamp(1, 64);
    (2 * s * i).clamp(4, 96)
}

fn rgb(c: &Color) -> Vec3 {
    Vec3::new(c.r, c.g, c.b)
}

/// og.WeatherState drawn: the lit surfaces' wetness, puddles and snow,
/// and the latest lightning strike for 0.6 s after its tick (a double
/// flash of the sky's light, a bolt from the cloud base to the ground and
/// a bright light where it lands). A pure function of the world's state.
fn lightning_and_surfaces(
    world: &World,
    scene: &mut og_render::RenderScene,
    layouts: &AtmoLayouts,
    tick: u64,
) {
    scene.atmosphere.surface_weather = None;
    let Some(st) = resource(world, layouts.weather_state.as_ref())
        .and_then(|st| record::WeatherState::from_json(&st).ok())
    else {
        return;
    };
    // The hand read's wind default (zero), not the IDL's (1.5, 0, 0.5).
    let weather = resource(world, layouts.weather.as_ref()).and_then(|w| {
        let base = record::Weather {
            wind: Vec3::ZERO,
            ..Default::default()
        };
        record::Weather::from_json_over(&w, base).ok()
    });
    let kind = weather.as_ref().map_or(0, |w| w.kind);
    let rain = weather
        .as_ref()
        .filter(|_| kind == og_render::WEATHER_RAIN)
        .map_or(0.0, |w| w.intensity);
    // The shelter map looks along the fall: the rain's or the snow's speed
    // (og_render::weather's particles) slanted by the wind, at most 45°
    // (what drifts flatter eddies round a roof, not under it); straight
    // down once neither falls.
    let (drift, down) = match (kind, weather.as_ref()) {
        (og_render::WEATHER_RAIN, Some(w)) => (w.wind, 11.0),
        (og_render::WEATHER_SNOW, Some(w)) => (w.wind * 0.8, 0.88),
        _ => (Vec3::ZERO, 1.0),
    };
    let drift = Vec3::new(drift.x, 0.0, drift.z).clamp_length_max(down);
    let fall = drift - Vec3::Y * down;
    scene.atmosphere.surface_weather = Some(og_render::SurfaceWeather {
        wetness: st.wetness,
        puddles: st.puddles,
        snow: st.snow_depth / st.snow_cover.max(1e-3),
        rain,
        fall: fall.normalize_or(Vec3::NEG_Y),
        shelter: st.shelter_size,
        dust: st.dust,
        dust_color: rgb(&st.dust_color),
        snow_depth: st.snow_depth,
        ..Default::default()
    });
    let strikes = st.strikes;
    if strikes == 0 {
        return;
    }
    let rate = f64::from(world.time().dt).recip();
    let since = (tick.saturating_sub(u64::from(st.last_strike))) as f64 / rate;
    let at = st.strike_position;
    let base = scene.atmosphere.clouds.map_or(600.0, |c| c.base.max(50.0));
    og_render::add_lightning(scene, at, at.y + base, strikes, since as f32);
}

/// The solve's coarse draw mesh; the renderer owns normals and refinement.
/// og.Water.seabed's coast for the frame (W1): what the tick floats bodies
/// on (`super::seabed::coast`, read the same way from the same components,
/// at the entity's world position as og-physics places the water), in the
/// frame's coordinates before the render origin moves them.
fn seabed(
    world: &World,
    terrains: &super::terrain::TerrainState,
    terrain_layout: Option<&super::codec::Layout>,
    entity: Entity,
    v: &Value,
    row: &record::Water,
    over_mesh: bool,
) -> Option<og_render::WaterShore> {
    if over_mesh {
        return None;
    }
    let r = super::physics_ext::Reader::World(world);
    let (position, _, _) = r.pose(entity)?;
    let sea = og_core::ocean::Ocean::new(row.wind, row.swell, row.choppiness);
    let (field, trains) =
        super::seabed::coast(terrains, terrain_layout, &r, v, position, sea.as_ref())?;
    Some(og_render::WaterShore {
        origin: field.origin,
        field,
        trains,
        foam: row.shore_foam,
    })
}

fn soft_mesh(points: &[Vec3], indices: &[u32]) -> MeshData {
    let vertices = points
        .iter()
        .map(|p| Vertex {
            tangent: [0.0; 4],
            position: p.to_array(),
            normal: [0.0; 3],
            ..Default::default()
        })
        .collect();
    MeshData {
        vertices,
        indices: indices.to_vec(),
    }
}

/// A coarse `columns` x `rows` grid of world points and UVs.
fn cloth_mesh(columns: u32, rows: u32, points: &[Vec3]) -> MeshData {
    let (w, ht) = (columns.max(2) as usize, rows.max(2) as usize);
    if points.len() < w * ht {
        return MeshData {
            vertices: Vec::new(),
            indices: Vec::new(),
        };
    }
    let at = |x: usize, y: usize| points[y * w + x];
    let mut vertices = Vec::with_capacity(w * ht);
    for y in 0..ht {
        for x in 0..w {
            vertices.push(Vertex {
                tangent: [0.0; 4],
                position: at(x, y).to_array(),
                normal: [0.0; 3],
                uv: [x as f32 / (w - 1) as f32, y as f32 / (ht - 1) as f32],
                ..Default::default()
            });
        }
    }
    let mut indices = Vec::with_capacity((w - 1) * (ht - 1) * 6);
    for y in 0..ht - 1 {
        for x in 0..w - 1 {
            let i = (y * w + x) as u32;
            let (r, d) = (i + 1, i + w as u32);
            indices.extend_from_slice(&[i, d, r, r, d, d + 1]);
        }
    }
    MeshData { vertices, indices }
}

// ── og.atmosphere.timeOfDay ─────────────────────────────────────────────

pub fn og_atmosphere_time_of_day(world: u32) -> f64 {
    let Ok(eng) = engine(world) else {
        return 0.0;
    };
    let Some(layout) = eng.state.borrow().layouts.get("og.DayNight").cloned() else {
        return 9.0;
    };
    // Inside a system, through its context (og.DayNight declared among its
    // reads); between ticks, from the world.
    if let Some(frame) = super::engine::running(world) {
        // SAFETY: the frame is published while this system runs.
        let ctx = unsafe { frame.ctx() };
        let time = ctx.time();
        let d = super::api::running_record(&eng, "og.DayNight", super::codec::RecordKind::Resource)
            .ok()
            .and_then(|(l, id)| {
                l.read::<record::DayNight>(&ctx.resource_bytes(id).ok()?)
                    .ok()
            });
        return d.map_or(9.0, |d| f64::from(hours(&d, time.tick, time.dt)));
    }
    let Ok(w) = eng.world() else {
        return 0.0;
    };
    let time = w.time();
    resource(w, Some(&layout))
        .and_then(|d| record::DayNight::from_json(&d).ok())
        .map_or(9.0, |d| f64::from(hours(&d, time.tick, time.dt)))
}

// ── material shaders ────────────────────────────────────────────────────

/// `og.load.shader`: a cooked shader, added to the scene once per path.
pub fn og_load_shader(world: u32, path: String) -> Result<serde_json::Value, String> {
    (|| -> Result<Value, String> {
        let eng = engine(world)?;
        Ok(json!(super::assets::handle(load_shader(&eng, &path)?)))
    })()
}

/// A cooked shader's load index, added to the scene once per path (an
/// effect material's hook loads it too, `super::effect`).
pub(crate) fn load_shader(eng: &super::engine::Engine, path: &str) -> Result<usize, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    if let Some(&i) = eng.render.borrow().atmo.shader_paths.get(&id) {
        return Ok(i);
    }
    let shader = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        bundle
            .shader(&id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    let mut rs = eng.render.borrow_mut();
    let h = rs.scene.add_shader(shader);
    rs.atmo.shaders.push(h);
    let i = rs.atmo.shaders.len() - 1;
    rs.atmo.shader_paths.insert(id, i);
    Ok(i)
}

pub fn og_render_shader_material(
    world: u32,
    shader: f64,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    (|| -> Result<Value, String> {
        let eng = engine(world)?;
        let desc = parse(desc.as_deref())?;
        let mut rs = eng.render.borrow_mut();
        let s = super::assets::index_of(shader)
            .and_then(|i| rs.atmo.shaders.get(i).copied())
            .ok_or_else(|| format!("shader {shader} is not loaded in this world"))?;
        let m = super::api::material(
            og_render::StandardMaterial::default(),
            &desc,
            &rs.textures,
            &rs.texture_arrays,
        )?;
        let handle = rs.scene.add_standard_material(m);
        rs.scene.set_material_shader(handle, Some(s));
        rs.materials.push(handle);
        Ok(json!(super::api::material_handle(rs.materials.len() - 1)))
    })()
}

/// og.render.setMaterialShader: `shader` is the JSON of a shader handle or null.
pub fn og_render_set_material_shader(
    world: u32,
    material: f64,
    shader: String,
) -> Result<(), String> {
    let r = (|| -> Result<(), String> {
        let eng = engine(world)?;
        let shader: Option<u64> = super::wire::arg(Some(&shader), "shader")?;
        let mut rs = eng.render.borrow_mut();
        let h = super::assets::index_of(material)
            .and_then(|i| rs.materials.get(i).copied())
            .ok_or_else(|| format!("material {material} is not alive"))?;
        let s = match shader {
            None => None,
            Some(handle) => Some(
                super::assets::index_of(handle as f64)
                    .and_then(|i| rs.atmo.shaders.get(i).copied())
                    .ok_or_else(|| format!("shader {handle} is not loaded in this world"))?,
            ),
        };
        if !rs.scene.set_material_shader(h, s) {
            return Err(format!("material {material} is not a standard material"));
        }
        Ok(())
    })();
    match r {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn og_render_set_shader_param(
    world: u32,
    material: f64,
    name: String,
    value: String,
) -> Result<(), String> {
    let r = (|| -> Result<(), String> {
        let eng = engine(world)?;
        let value: Vec<f32> = match serde_json::from_str::<Value>(&value) {
            // Hand read: a bare number, which the IDL's List.f32 does not hold.
            Ok(Value::Number(n)) => vec![n.as_f64().unwrap_or(0.0) as f32],
            Ok(v @ Value::Array(_)) => super::wire::value(&v, "value")?,
            _ => return Err("value is a list of numbers".into()),
        };
        let mut rs = eng.render.borrow_mut();
        let h = super::assets::index_of(material)
            .and_then(|i| rs.materials.get(i).copied())
            .ok_or_else(|| format!("material {material} is not alive"))?;
        if rs.scene.material_shader(h).is_none() {
            return Err(format!(
                "material {material} has no shader (make it with og.render.shaderMaterial or give it one with og.render.setMaterialShader)"
            ));
        }
        if !rs.scene.set_shader_param(h, &name, &value) {
            return Err(format!(
                "the material's shader declares no parameter '{name}'"
            ));
        }
        Ok(())
    })();
    match r {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}
