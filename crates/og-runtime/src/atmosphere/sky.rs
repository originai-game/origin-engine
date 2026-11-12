//! The sky as the world declares it: og.PhysicalSky, og.Sky, og.DayNight
//! (the clock, the light and the sky it makes), og.Clouds and og.Aurora
//! into the scene's light, environment and background.

use super::{AtmoLayouts, rgb};
use crate::codec::Layout;
use crate::engine::engine;
use crate::fx::resource;
use crate::wind::WorldWind;
use crate::wire::{FromJson, record};
use glam::Vec3;
#[cfg(not(feature = "publish"))]
use og_ecs::Entity;
use og_ecs::World;
use og_render::{Background, ProceduralSky, SkyLook};
#[cfg(not(feature = "publish"))]
use serde_json::{Value, json};
#[cfg(not(feature = "publish"))]
use std::collections::BTreeMap;

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

/// The fog's tint from the day-night cycle (its colour, and its sun's
/// when the cycle gives one), for the fog drawn after the sky.
pub(super) type FogTint = Option<(Vec3, Option<Vec3>)>;

/// The sky, its light and its clouds into `scene`.
pub(super) fn extract(
    world: &World,
    scene: &mut og_render::RenderScene,
    layouts: &AtmoLayouts,
    world_wind: WorldWind,
) -> FogTint {
    // No twilight adaptation unless the day-night cycle sets one below.
    scene.atmosphere.adaptation = 1.0;
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
        .and_then(|c| record::Clouds::from_json(&c).ok())
        .filter(|c| c.enabled)
        .map(|c| og_render::Clouds {
            coverage: c.coverage,
            density: c.density,
            base: c.base,
            thickness: c.thickness,
            scale: c.scale,
            wind: c.wind + world_wind.aloft(),
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
    fog_tint
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
            let first = crate::fx::rows(world, layouts.get("og.DirectionalLight"))
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
    if let Some(frame) = crate::engine::running(world) {
        // SAFETY: the frame is published while this system runs.
        let ctx = unsafe { frame.ctx() };
        let time = ctx.time();
        let d = crate::api::running_record(&eng, "og.DayNight", crate::codec::RecordKind::Resource)
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
