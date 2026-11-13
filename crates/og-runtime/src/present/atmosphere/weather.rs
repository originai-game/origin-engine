//! The weather drawn: og.Fog (plain or volumetric), og.Weather's
//! particles, fog and dimming, and og.WeatherState's wet, snowed and
//! dusted surfaces and lightning. The state itself steps in the tick
//! (`crate::bind::weather`).

use super::sky::FogTint;
use super::{AtmoLayouts, AtmoState, rgb};
use crate::bind::wind::WorldWind;
use crate::host::wire::{FromJson, record};
use crate::present::fx::resource;
use glam::{Mat4, Vec3};
use og_ecs::World;
use og_render::{Fog, ParticleSystem, Projection};

/// The particle system id of the weather (no entity has it).
const WEATHER_ID: u64 = u64::MAX - 0x5745;

/// Fog and weather into `scene`, over the sky already there.
pub(super) fn extract(
    world: &World,
    scene: &mut og_render::RenderScene,
    layouts: &AtmoLayouts,
    atmo: &mut AtmoState,
    world_wind: WorldWind,
    fog_tint: FogTint,
) {
    scene.atmosphere.fog = None;
    scene.atmosphere.volumetric_fog = None;
    let tick = world.time().tick;
    let orthographic = matches!(
        scene.frame.camera.projection,
        Projection::Orthographic { .. }
    );
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
    let weather =
        resource(world, layouts.weather.as_ref()).and_then(|w| record::Weather::from_json(&w).ok());
    if weather.is_none()
        && let Some(c) = scene.atmosphere.clouds
    {
        c.light_scene(scene);
    }
    let mut falling = None;
    if let Some(w) = weather {
        let kind = w.kind;
        let layer = orthographic.then_some(w.layer);
        let wind = w.wind + world_wind.velocity;
        let look = og_render::weather(
            kind,
            w.intensity,
            wind,
            w.radius,
            rgb(&w.tint),
            w.dim,
            layer,
        );
        // The sky closes to the weather's cover and the light dims once;
        // a deck the weather brought drifts on the world's wind.
        let declared = scene.atmosphere.clouds.is_some();
        look.light_scene(scene);
        if let Some(deck) = scene.atmosphere.clouds.as_mut().filter(|_| !declared) {
            deck.wind += world_wind.aloft();
        }
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
        if let Some(emitter) = &look.emitter {
            // The emitter is set again only when its inputs change (its
            // intensity changes its emission alone, so the particles in
            // the air stay).
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
        falling = Some(look);
    }

    lightning_and_surfaces(world, scene, layouts, tick, falling.as_ref());
}

/// og.WeatherState drawn: the lit surfaces' wetness, puddles and snow,
/// and the latest lightning strike for 0.6 s after its tick (a double
/// flash of the sky's light, a bolt from the cloud base to the ground and
/// a bright light where it lands). The surfaces lie under `falling`, the
/// weather now (its rain ripples the puddles, the shelter looks along its
/// fall). A pure function of the world's state.
fn lightning_and_surfaces(
    world: &World,
    scene: &mut og_render::RenderScene,
    layouts: &AtmoLayouts,
    tick: u64,
    falling: Option<&og_render::WeatherLook>,
) {
    scene.atmosphere.surface_weather = None;
    let Some(st) = resource(world, layouts.weather_state.as_ref())
        .and_then(|st| record::WeatherState::from_json(&st).ok())
    else {
        return;
    };
    let lying = og_render::SurfaceWeather {
        wetness: st.wetness,
        puddles: st.puddles,
        snow: st.snow_depth / st.snow_cover.max(1e-3),
        shelter: st.shelter_size,
        dust: st.dust,
        dust_color: rgb(&st.dust_color),
        snow_depth: st.snow_depth,
        ..Default::default()
    };
    scene.atmosphere.surface_weather = Some(falling.map_or(lying, |look| lying.under(look)));
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
