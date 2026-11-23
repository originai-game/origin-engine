//! The sky as the world declares it: og.PhysicalSky, og.Sky, og.DayNight
//! (the clock, the light and the sky it makes), og.Clouds and og.Aurora
//! into the scene's light, environment and background.

use super::{AtmoLayouts, rgb};
use crate::bind::season::Now as SeasonNow;
use crate::bind::wind::WorldWind;
use crate::engine::engine;
use crate::host::codec::Layout;
use crate::host::wire::{self, FromJson, record};
use crate::present::fx::resource;
use glam::Vec3;
#[cfg(not(feature = "publish"))]
use og_ecs::Entity;
use og_ecs::World;
use og_render::{Background, ProceduralSky, SkyLook};
#[cfg(not(feature = "publish"))]
use serde_json::{Value, json};
#[cfg(not(feature = "publish"))]
use std::collections::BTreeMap;

/// The DayNight clock at `time`'s tick (hours, 0..24), from its row.
fn hours(d: &record::DayNight, time: &og_core::Time) -> f64 {
    og_env::day::hours(d.start, d.speed, d.enabled, time.elapsed())
}

/// An enabled og.DayNight at the world's tick: its row, the clock, the
/// look, and lightIntensity, the multiplier on its sun and moon.
struct DayNightNow {
    d: record::DayNight,
    /// The og.Season that sets the sun's path, if one does.
    season: Option<SeasonNow>,
    hours: f64,
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
    season: Option<&Layout>,
) -> Option<DayNightNow> {
    let d = record::DayNight::from_json(&*resource(world, layout)?).ok()?;
    if !d.enabled {
        return None;
    }
    let time = world.time();
    let now = hours(&d, &time);
    let air = physical_air(world, physical);
    let season = season_sun(world, season);
    let look = look_at(&d, season.as_ref(), now);
    let look = match &air {
        Some(a) => look.with_atmosphere(a),
        None => look,
    };
    let intensity = d.light_intensity.max(0.0);
    Some(DayNightNow {
        d,
        season,
        hours: now,
        look,
        intensity,
    })
}

/// og.DayNight's look at the world's tick and the multiplier on its sun
/// and moon (lightIntensity): the key light's candidates
/// (`crate::present::key_light`). `None` while it is absent or off.
pub(crate) fn day_night_look(
    world: &World,
    layout: Option<&Layout>,
    physical: Option<&Layout>,
    season: Option<&Layout>,
) -> Option<(og_render::DayNightLook, f32)> {
    day_night_now(world, layout, physical, season).map(|n| (n.look, n.intensity))
}

/// An enabled og.Season that sets the sun's path (Season.sun), at the
/// world's tick.
fn season_sun(world: &World, layout: Option<&Layout>) -> Option<SeasonNow> {
    let s = record::Season::from_json(&*resource(world, layout)?).ok()?;
    let time = world.time();
    Some(SeasonNow::of(&s, time.elapsed())).filter(|s| s.sun)
}

/// The year an enabled og.Season draws the seasonal leaves in, at the
/// world's tick: its climate's and the date as the place lives it.
fn leaf_calendar(
    world: &World,
    layout: Option<&Layout>,
) -> Option<og_render::foliage::LeafCalendar> {
    let s = record::Season::from_json(&*resource(world, layout)?).ok()?;
    if !s.enabled {
        return None;
    }
    let time = world.time();
    let now = SeasonNow::of(&s, time.elapsed());
    Some(og_render::foliage::LeafCalendar {
        year: og_env::season::leaf_year(&now.climate),
        day: og_env::season::local(now.day, now.latitude),
    })
}

/// The cycle's look at `hours`, with its moon: under the date's sun at the latitude when
/// an og.Season sets the path, else along og.DayNight's own. Either way
/// DayNight.azimuth is the heading of due east, and the noon sun of the
/// north stands a quarter turn on from it (the heading azimuth + 90, the
/// way `tilt` leans the path).
fn look_at(
    d: &record::DayNight,
    season: Option<&SeasonNow>,
    hours: f64,
) -> og_render::DayNightLook {
    // The sun's geometry is f32 arithmetic: the clock is rounded once here.
    let hours = hours as f32;
    let look = match season {
        Some(s) => {
            let [east, up, south] = s.sun(hours);
            let turn = glam::Quat::from_rotation_y((d.azimuth - 90.0).to_radians());
            og_render::day_night_under(turn * Vec3::new(east, up, -south))
        }
        None => og_render::day_night(hours, d.tilt, d.azimuth),
    };
    // The moon opposite the sun (moon_direction zero).
    look.with_moon(d.moon_size, d.moon_phase)
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
    scene.atmosphere.leaves = leaf_calendar(world, layouts.season.as_ref());
    // The physical sky: alone, towards the directional light drawn.
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
            sun_size: SkyLook::PHYSICAL_SUN_SIZE,
            sun_intensity: SkyLook::PHYSICAL_SUN_INTENSITY,
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
        && let Ok(s) = record::Sky::from_json(&s)
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
        // A zero sunDirection follows the light the frame is lit by (the
        // directional light `lights` drew, og_render::key_light; the
        // built-in sun with none): the sun stands where the light comes from.
        let dir = s
            .sun_direction
            .try_normalize()
            .or_else(|| scene.lighting.light.direction_to_light.try_normalize())
            .unwrap_or(Vec3::Y);
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
        season,
        hours: now,
        look,
        intensity: _,
    }) = day_night_now(
        world,
        layouts.day_night.as_ref(),
        layouts.physical_sky.as_ref(),
        layouts.season.as_ref(),
    ) {
        // The image-based light steps every `step` game minutes.
        let step = f64::from(d.step.clamp(1.0, 120.0)) / 60.0;
        let mut stepped = look_at(&d, season.as_ref(), (now / step).floor() * step);
        if let Some(a) = &air {
            stepped = stepped.with_atmosphere(a);
        }
        // The light itself is the key light's (`lights`, which takes the
        // cycle's sun and moon among its candidates).
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
            kind: match c.kind {
                wire::CloudKind::Cumulus => og_render::CloudKind::Cumulus,
                wire::CloudKind::Stratus => og_render::CloudKind::Stratus,
                wire::CloudKind::Stratocumulus => og_render::CloudKind::Stratocumulus,
                wire::CloudKind::Cumulonimbus => og_render::CloudKind::Cumulonimbus,
            },
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
            shadows: c.shadows,
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
/// (`hours`, `toSun`) and the light drawn (`toLight`, `lightColor`,
/// `lightIntensity`: the key light, `crate::present::key_light`, of which
/// the cycle's sun and moon are candidates); the og.DirectionalLight it
/// drives (the one the key light would draw without it) is answered with
/// the colour and intensity it is drawn with, `toLight` (the unit vector
/// towards the light drawn, which replaces the entity's -Z) and
/// `drivenBy`; every og.DirectionalLight gains `keyShare`, its share of
/// the light drawn (1 drawn alone, between 0 and 1 while two hand over, 0
/// left out); og.Season is answered with og.atmosphere.season's fields,
/// its `day` the date drawn and the air's temperature now as
/// `temperatureNow` (og.Season.temperature is the year's mean). Reads
/// only; the world is never written. A publish build has no `get`, so no
/// derive.
#[cfg(not(feature = "publish"))]
pub(crate) fn derive(
    world: &World,
    layouts: &BTreeMap<String, Layout>,
    baked: &crate::present::baked::BakedState,
    entity: Option<Entity>,
    name: &str,
    fields: &mut serde_json::Map<String, Value>,
) {
    if let (crate::bind::season::STATE, None) = (name, entity) {
        use crate::host::wire::ToJson;
        if let Some(info) = crate::bind::season::of_world(world, layouts)
            && let Value::Object(info) = info.to_json()
        {
            fields.extend(info.into_iter().map(|(k, v)| match k.as_str() {
                "temperature" => ("temperatureNow".into(), v),
                _ => (k, v),
            }));
        }
        return;
    }
    if !matches!(
        (name, entity),
        ("og.DayNight", None) | ("og.DirectionalLight", Some(_))
    ) {
        return;
    }
    let now = day_night_now(
        world,
        layouts.get("og.DayNight"),
        layouts.get("og.PhysicalSky"),
        layouts.get(crate::bind::season::STATE),
    );
    if now.is_none() && entity.is_none() {
        return;
    }
    let stage = crate::present::key_light::Layouts {
        directional: layouts.get("og.DirectionalLight"),
        day_night: layouts.get("og.DayNight"),
        physical_sky: layouts.get("og.PhysicalSky"),
        season: layouts.get(crate::bind::season::STATE),
    };
    let mut h = crate::present::extract::Hierarchy::of(world);
    let Some(key) = crate::present::key_light::key(
        world,
        stage,
        &mut |e| h.world_of(e),
        og_render::DirectionalLight::default(),
        baked,
    ) else {
        return;
    };
    let light = key.light;
    let to_light = light.direction_to_light.normalize_or_zero();
    match entity {
        Some(e) => {
            fields.insert("keyShare".into(), json!(key.share_of(e.to_bits())));
            if key.driven != Some(e.to_bits()) {
                return;
            }
            // The stored colour's lanes (a fourth, if any, kept).
            let mut color = fields.get("color").cloned().unwrap_or(json!([1, 1, 1]));
            if let Some(lanes) = color.as_array_mut() {
                for (i, v) in light.color.to_array().into_iter().enumerate() {
                    if let Some(slot) = lanes.get_mut(i) {
                        *slot = json!(v);
                    }
                }
            }
            fields.insert("color".into(), color);
            fields.insert("intensity".into(), json!(light.intensity));
            fields.insert("toLight".into(), xyz(to_light));
            fields.insert("drivenBy".into(), json!("og.DayNight"));
        }
        None => {
            let Some(now) = now else { return };
            fields.insert("hours".into(), json!(now.hours));
            fields.insert("toSun".into(), xyz(now.look.sun_direction));
            fields.insert("toLight".into(), xyz(to_light));
            fields.insert("lightColor".into(), xyz(light.color));
            fields.insert("lightIntensity".into(), json!(light.intensity));
        }
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
    // reads; undeclared, refused and logged once: NaN, never a made-up
    // hour); between ticks, from the world.
    if let Some(frame) = crate::engine::running(world) {
        // SAFETY: the frame is published while this system runs.
        let ctx = unsafe { frame.ctx() };
        let time = ctx.time();
        let Ok((l, id)) = crate::host::api::running_record(
            &eng,
            "og.DayNight",
            crate::host::codec::RecordKind::Resource,
        ) else {
            return 9.0;
        };
        let d = match ctx.resource_bytes(id) {
            Ok(bytes) => l.read::<record::DayNight>(&bytes).ok(),
            Err(e) if e.kind() == og_core::ErrorKind::Access => {
                crate::host::api::undeclared(&eng, frame.system, "read", "og.DayNight");
                return f64::NAN;
            }
            Err(_) => None,
        };
        return d.map_or(9.0, |d| hours(&d, &time));
    }
    let Ok(w) = eng.world() else {
        return 0.0;
    };
    let time = w.time();
    resource(w, Some(&layout))
        .and_then(|d| record::DayNight::from_json(&d).ok())
        .map_or(9.0, |d| hours(&d, &time))
}
