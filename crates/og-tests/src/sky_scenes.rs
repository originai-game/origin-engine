//! The realism wave's sky and weather scenes (R4): the physical sky over
//! the colonnade at noon, at sunset looking into the sun, in twilight after
//! it has set, and on a hazy afternoon; clouds; wet ground with puddles,
//! lying snow and a lightning strike; a canopy keeping the snow and the
//! rain off the ground under it.

use crate::atmo_scenes::{colonnade, colonnade_camera};
use glam::{Mat4, Vec3};
use og_render::{
    Background, Bloom, Clouds, DirectionalLight, Environment, Exposure, Fog, Material,
    PhysicalAtmosphere, ProceduralSky, RenderScene, SurfaceWeather, day_night, shapes,
};

/// The scenes of this module, in `SCENES` order.
pub const SKY_SCENES: &[&str] = &[
    "sky-physical-noon",
    "sky-physical-sunset",
    "sky-physical-twilight",
    "sky-physical-hazy",
    "clouds-fair",
    "clouds-overcast",
    "clouds-sunset",
    "clouds-gradient",
    "weather-wet",
    "weather-snow-cover",
    "weather-lightning",
    "weather-shelter-snow",
    "weather-shelter-rain",
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
        sun_color: look.fog_sun.unwrap_or(Fog::default().sun_color),
        density: 0.0015,
        start: 12.0,
        sky: 0.0,
        ..Default::default()
    });
    scene.post.bloom = Some(Bloom::default());
    scene.camera = colonnade_camera();
    if into_sun {
        let s = look.sun_direction;
        let bearing = Vec3::new(s.x, 0.0, s.z).normalize_or(Vec3::NEG_Z);
        // Tilted up with the sun (a 15° pitch at most), so a sun up to
        // about 40° high stands in the frame with its glow.
        let lift = (s.y.max(0.0).asin() - 0.05).clamp(0.07, 0.26).tan();
        scene.camera.target = scene.camera.position + bearing * 40.0 + Vec3::Y * 40.0 * lift;
    }
    scene
}

/// `scene` under `clouds`, the light dimmed by their cover as the runtime
/// does, the camera tilted up to the sky.
fn clouded(mut scene: RenderScene, clouds: Clouds) -> RenderScene {
    let (sun, sky) = clouds.light();
    scene.light.intensity *= sun;
    scene.environment.intensity *= sky;
    scene.clouds = Some(clouds);
    scene.camera.target += Vec3::Y * 14.0;
    scene
}

/// The colonnade under a grey overcast at `hours` with `weather` on its
/// surfaces, the camera lowered to look along the ground.
fn weathered(hours: f32, weather: SurfaceWeather) -> RenderScene {
    let mut scene = clouded(
        physical_sky_scene(hours, PhysicalAtmosphere::EARTH, false),
        Clouds {
            coverage: 0.9,
            color: Vec3::splat(0.75),
            base: 900.0,
            layer: 0.0,
            ..Clouds::default()
        },
    );
    scene.surface_weather = Some(weather);
    scene.camera.position = Vec3::new(1.5, 1.6, 4.0);
    scene.camera.target = Vec3::new(-1.0, 0.2, -30.0);
    scene
}

/// A flat canopy 8 m wide (x -4 to 4) and 6 m deep (z -9 to -3), its
/// underside 3 m up, on four posts: in front of the weathered camera, which
/// sees the ground under it.
pub fn canopy(scene: &mut RenderScene) {
    let slab = scene.add_mesh(shapes::cuboid(Vec3::new(8.0, 0.2, 6.0)));
    let post = scene.add_mesh(shapes::cuboid(Vec3::new(0.2, 3.0, 0.2)));
    let wood = scene.add_material(Material {
        base_color: [0.42, 0.3, 0.2, 1.0],
        roughness: 0.8,
        ..Default::default()
    });
    scene.add_instance(
        slab,
        wood,
        Mat4::from_translation(Vec3::new(0.0, 3.1, -6.0)),
    );
    for x in [-3.8f32, 3.8] {
        for z in [-8.8f32, -3.2] {
            scene.add_instance(post, wood, Mat4::from_translation(Vec3::new(x, 1.5, z)));
        }
    }
}

/// `scene` with its camera's exposure opened by `ev` stops.
fn exposed(mut scene: RenderScene, ev: f32) -> RenderScene {
    if let Exposure::Manual { ev: e } = &mut scene.camera.exposure {
        *e += ev;
    }
    scene
}

/// The wet colonnade on a stormy night, 0.03 s after a strike 160 m
/// ahead (`strike`), or the same night without it.
pub fn storm(strike: bool) -> RenderScene {
    let mut scene = weathered(
        22.5,
        SurfaceWeather {
            wetness: 1.0,
            puddles: 0.5,
            rain: 1.0,
            ..SurfaceWeather::default()
        },
    );
    scene.camera.target = Vec3::new(-1.0, 8.0, -30.0);
    if strike {
        let at = Vec3::new(-25.0, 0.0, -160.0);
        og_render::add_lightning(&mut scene, at, 900.0, 3, 0.03);
    }
    scene
}

/// The scene called `name` of [`SKY_SCENES`].
pub fn sky_scene(name: &str) -> RenderScene {
    let earth = PhysicalAtmosphere::EARTH;
    match name {
        "weather-wet" => weathered(
            11.0,
            SurfaceWeather {
                wetness: 1.0,
                puddles: 0.7,
                rain: 1.0,
                ..SurfaceWeather::default()
            },
        ),
        "weather-snow-cover" => weathered(
            11.0,
            SurfaceWeather {
                snow: 0.6,
                ..SurfaceWeather::default()
            },
        ),
        "weather-lightning" => storm(true),
        "weather-shelter-snow" => {
            let mut scene = weathered(
                11.0,
                SurfaceWeather {
                    snow: 0.8,
                    ..SurfaceWeather::default()
                },
            );
            canopy(&mut scene);
            scene
        }
        "weather-shelter-rain" => {
            // The rain slanted by a wind from -x: the dry patch shifts
            // downwind (+x) of the canopy.
            let mut scene = weathered(
                11.0,
                SurfaceWeather {
                    wetness: 1.0,
                    puddles: 0.8,
                    rain: 1.0,
                    fall: Vec3::new(0.35, -1.0, 0.0).normalize(),
                    ..SurfaceWeather::default()
                },
            );
            canopy(&mut scene);
            scene
        }
        "clouds-fair" => clouded(
            physical_sky_scene(11.0, earth, false),
            Clouds {
                coverage: 0.4,
                ..Clouds::default()
            },
        ),
        "clouds-overcast" => clouded(
            physical_sky_scene(13.0, earth, false),
            Clouds {
                coverage: 0.95,
                density: 0.06,
                color: Vec3::splat(0.8),
                base: 900.0,
                layer: 0.0,
                ..Clouds::default()
            },
        ),
        "clouds-sunset" => clouded(
            physical_sky_scene(17.7, earth, true),
            Clouds {
                coverage: 0.5,
                layer: 0.6,
                ..Clouds::default()
            },
        ),
        "clouds-gradient" => clouded(
            crate::atmo_scenes::day_night_scene(10.5, false),
            Clouds::default(),
        ),
        "sky-physical-noon" => physical_sky_scene(12.5, earth, false),
        // A camera opens up for the low light as an eye adapts: half a stop
        // into the setting sun (more bleaches its glow yellow-white), 3.5
        // in twilight (the sky's light is ten stops below noon: opened
        // less, it still reads as dusk).
        "sky-physical-sunset" => exposed(physical_sky_scene(17.85, earth, true), 0.5),
        "sky-physical-twilight" => exposed(physical_sky_scene(18.45, earth, true), 3.5),
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
