//! The realism wave's sky and weather scenes (R4): the physical sky over
//! the colonnade at noon, at sunset looking into the sun, in twilight after
//! it has set, and on a hazy afternoon.

use crate::atmo_scenes::{colonnade, colonnade_camera};
use glam::Vec3;
use og_render::{
    Background, Bloom, DirectionalLight, Environment, Fog, PhysicalAtmosphere, ProceduralSky,
    RenderScene, day_night,
};

/// The scenes of this module, in `SCENES` order.
pub const SKY_SCENES: &[&str] = &[
    "sky-physical-noon",
    "sky-physical-sunset",
    "sky-physical-twilight",
    "sky-physical-hazy",
];

/// The colonnade at `hours` under the physical sky of `air`, lit by the
/// day-night look made physical (`DayNightLook::with_atmosphere`), with a
/// light fog of the sky's horizon colour. `into_sun`: the camera turns to
/// face the sun's bearing.
pub fn physical_sky_scene(hours: f32, air: PhysicalAtmosphere, into_sun: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    colonnade(&mut scene);
    let look = day_night(hours, 25.0, -20.0).with_atmosphere(&air);
    scene.light = DirectionalLight {
        direction_to_light: look.light_direction,
        color: look.light_color,
        intensity: look.light_intensity,
        ..Default::default()
    };
    scene.environment = Environment {
        sky: ProceduralSky {
            sun_direction: look.sun_direction,
            sun_color: look.sun_color,
            look: Some(look.sky),
        },
        intensity: look.ambient,
        background: Background::Procedural {
            sun_direction: look.sun_direction,
            sun_color: look.sun_color,
        },
    };
    scene.fog = Some(Fog {
        color: look.fog_color,
        density: 0.006,
        start: 12.0,
        sky: 0.0,
        ..Default::default()
    });
    scene.post.bloom = Some(Bloom::default());
    scene.camera = colonnade_camera();
    if into_sun {
        let s = look.sun_direction;
        let bearing = Vec3::new(s.x, 0.0, s.z).normalize_or(Vec3::NEG_Z);
        scene.camera.target = scene.camera.position + bearing * 40.0 + Vec3::Y * 4.0;
    }
    scene
}

/// The scene called `name` of [`SKY_SCENES`].
pub fn sky_scene(name: &str) -> RenderScene {
    let earth = PhysicalAtmosphere::EARTH;
    match name {
        "sky-physical-noon" => physical_sky_scene(12.5, earth, false),
        "sky-physical-sunset" => physical_sky_scene(17.85, earth, true),
        "sky-physical-twilight" => physical_sky_scene(18.45, earth, true),
        _ => physical_sky_scene(
            15.5,
            PhysicalAtmosphere {
                mie: earth.mie * 8.0,
                mie_absorption: earth.mie_absorption * 8.0,
                ..earth
            },
            true,
        ),
    }
}
