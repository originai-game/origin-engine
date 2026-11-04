//! The realism wave's sky and weather scenes (R4): the physical sky over
//! the colonnade at noon, at sunset looking into the sun, in twilight after
//! it has set, and on a hazy afternoon; clouds; wet ground with puddles,
//! lying snow and a lightning strike; a canopy keeping the snow and the
//! rain off the ground under it; the aurora on a clear night, and in a
//! substorm behind broken cloud.

use crate::atmo_scenes::{colonnade, colonnade_camera};
use glam::{Mat4, Vec3};
use og_render::{
    Aurora, Background, Bloom, Clouds, DirectionalLight, Environment, Exposure, Fog, Material,
    PhysicalAtmosphere, ProceduralSky, RenderScene, StandardMaterial, SurfaceWeather, day_night,
    shapes,
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
    "weather-shelter-meshlet",
    "aurora-arcs",
    "aurora-breakup",
    "live-sky",
];

/// The colonnade at `hours` under the physical sky of `air`, lit by the
/// day-night look made physical (`DayNightLook::with_atmosphere`), with a
/// light fog of the sky's horizon colour. `into_sun`: the camera turns to
/// face the sun's bearing.
pub fn physical_sky_scene(hours: f32, air: PhysicalAtmosphere, into_sun: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    colonnade(&mut scene);
    let look = physical_light(&mut scene, hours, air);
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

/// `scene` lit at `hours` under the physical sky of `air` as
/// [`physical_sky_scene`] lights the colonnade: the light, the sky and its
/// ambient, a light fog of the sky's colour, bloom. Returns the look.
pub fn physical_light(
    scene: &mut RenderScene,
    hours: f32,
    air: PhysicalAtmosphere,
) -> og_render::DayNightLook {
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
        ..Environment::default()
    };
    scene.adaptation = look.adaptation;
    scene.fog = Some(Fog {
        color: look.fog_color,
        sun_color: look.fog_sun.unwrap_or(Fog::default().sun_color),
        density: 0.0015,
        start: 12.0,
        sky: 0.0,
        ..Default::default()
    });
    scene.post.bloom = Some(Bloom::default());
    look
}

/// `live-sky`: a chrome sphere on a grey ground at 14:00 under the
/// physical sky, seen from a little above, under an overcast deck of
/// rose-tinted cloud (no light dimmed: only the sky changes); without
/// `clouds` the same deck at no coverage, so the two differ only in the
/// clouds' uniform. The
/// sphere's upper half reflects the sky over and behind the camera, out
/// of the frame, so only the live sky capture can bring the clouds into it.
pub fn live_sky_scene(clouds: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    physical_light(&mut scene, 14.0, PhysicalAtmosphere::EARTH);
    scene.fog = None;
    let ground = scene.add_mesh(shapes::plane(200.0, 1.0));
    let grey = scene.add_standard_material(StandardMaterial {
        color: Vec3::splat(0.35),
        roughness: 0.9,
        ..Default::default()
    });
    scene.add_instance(ground, grey, Mat4::IDENTITY);
    let sphere = scene.add_mesh(shapes::sphere(64, 32));
    let chrome = scene.add_standard_material(StandardMaterial {
        color: Vec3::splat(0.95),
        metalness: 1.0,
        roughness: 0.02,
        ..Default::default()
    });
    let at = Vec3::new(0.0, 1.0, 0.0);
    scene.add_instance(sphere, chrome, Mat4::from_translation(at));
    scene.camera = colonnade_camera();
    scene.camera.position = Vec3::new(0.0, 2.2, 4.5);
    scene.camera.target = at;
    scene.clouds = Some(Clouds {
        coverage: if clouds { 0.95 } else { 0.0 },
        density: 0.06,
        color: Vec3::new(1.0, 0.45, 0.5),
        base: 900.0,
        layer: 0.0,
        ..Clouds::default()
    });
    scene
}

/// `scene` under `clouds`, the light dimmed by their cover as the runtime
/// does, the camera tilted up to the sky.
fn clouded(mut scene: RenderScene, clouds: Clouds) -> RenderScene {
    clouds.light_scene(&mut scene);
    // A camera opens up under a deck that hides the sun by day, as a
    // photographer's meter does (1.5 stops for a closed overcast at noon,
    // as the deck closes over 0.6 to 0.95 cover): at the clear sky's
    // exposure a noon overcast read as dusk (its sky 0.32 in sRGB).
    let t = ((clouds.coverage - 0.6) / 0.35).clamp(0.0, 1.0);
    if scene.environment.sky.sun_direction.y > 0.0 {
        scene = exposed(scene, 1.5 * t * t * (3.0 - 2.0 * t));
    }
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
/// `clustered`: its slab and posts are clustered meshes, drawn through
/// the meshlet path (one level, a cluster per 64 triangles).
pub fn canopy(scene: &mut RenderScene, clustered: bool) {
    let mut mesh = |data: og_render::MeshData| {
        let clusters = clustered.then(|| one_level(&data));
        let m = scene.add_mesh(data);
        if clusters.is_some() {
            scene.set_mesh_clusters(m, clusters);
        }
        m
    };
    let slab = mesh(shapes::cuboid(Vec3::new(8.0, 0.2, 6.0)));
    let post = mesh(shapes::cuboid(Vec3::new(0.2, 3.0, 0.2)));
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

/// `data` as one level of clusters, each of up to 64 of its triangles in
/// order (their bounds from their vertices; a cone that never culls).
fn one_level(data: &og_render::MeshData) -> og_render::meshlet::MeshClusters {
    use og_render::meshlet::{CLUSTER_TRIANGLES, Cluster, ClusterLevel, MeshClusters};
    let pos = |i: &u32| Vec3::from(data.vertices[*i as usize].position);
    let clusters: Vec<Cluster> = data
        .indices
        .chunks(CLUSTER_TRIANGLES * 3)
        .enumerate()
        .map(|(k, ix)| {
            let (lo, hi) = ix
                .iter()
                .map(pos)
                .fold((Vec3::MAX, Vec3::MIN), |(a, b), p| (a.min(p), b.max(p)));
            let center = (lo + hi) * 0.5;
            Cluster {
                first: (k * CLUSTER_TRIANGLES * 3) as u32,
                triangles: (ix.len() / 3) as u32,
                center,
                radius: ix
                    .iter()
                    .map(|i| pos(i).distance(center))
                    .fold(0.0, f32::max),
                cone_axis: Vec3::Y,
                cone_cutoff: 1.0,
                error_center: center,
                error_radius: 1.0,
                ..Default::default()
            }
        })
        .collect();
    MeshClusters {
        levels: vec![ClusterLevel {
            first: 0,
            count: data.indices.len() as u32,
            error: 0.0,
            clusters: 0..clusters.len() as u32,
        }],
        clusters,
        indices: data.indices.clone(),
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
        // 1.5 km out, so the bolt's root in the deck (35° up) is in frame
        // and the glow about it reads.
        let at = Vec3::new(-150.0, 0.0, -1500.0);
        // The bolt is a ribbon in the scene: past the default 1 km far
        // plane it would be clipped.
        scene.camera.far = 4000.0;
        og_render::add_lightning(&mut scene, at, 900.0, 3, 0.03);
    }
    scene
}

/// The colonnade on a clear moonless night under `aurora` (its arcs out
/// along -z, where the camera looks), the camera tilted up and opened up
/// three stops for the night.
fn auroral(aurora: Aurora) -> RenderScene {
    let mut scene = exposed(
        physical_sky_scene(23.5, PhysicalAtmosphere::EARTH, false),
        3.0,
    );
    if let Some(look) = scene.environment.sky.look.as_mut() {
        look.moon_size = 0.0;
    }
    scene.aurora = Some(aurora);
    scene.camera.target += Vec3::Y * 22.0;
    scene
}

/// The scene called `name` of [`SKY_SCENES`].
pub fn sky_scene(name: &str) -> RenderScene {
    let earth = PhysicalAtmosphere::EARTH;
    match name {
        "live-sky" => live_sky_scene(true),
        // live-sky with its clouds cleared (no reference; the live-sky
        // check compares against it).
        "live-sky-clear" => live_sky_scene(false),
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
            canopy(&mut scene, false);
            scene
        }
        // The snow's shelter under a canopy of clustered meshes (the
        // meshlet path draws them into the weather map through a shadow
        // view of its dispatch).
        "weather-shelter-meshlet" => {
            let mut scene = weathered(
                11.0,
                SurfaceWeather {
                    snow: 0.8,
                    ..SurfaceWeather::default()
                },
            );
            canopy(&mut scene, true);
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
            canopy(&mut scene, false);
            scene
        }
        // clouds-fair without its clouds (no reference; the cumulus check
        // compares against it).
        "clouds-fair-clear" => clouded(
            physical_sky_scene(11.0, earth, false),
            Clouds {
                coverage: 0.0,
                layer: 0.0,
                ..Clouds::default()
            },
        ),
        "clouds-fair" => clouded(
            physical_sky_scene(11.0, earth, false),
            Clouds {
                coverage: 0.5,
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
        // Quiet folded arcs; and their night without them (no reference;
        // the check compares against it).
        "aurora-arcs" => auroral(Aurora {
            activity: 0.3,
            ..Aurora::default()
        }),
        "aurora-arcs-clear" => {
            let mut scene = auroral(Aurora::default());
            scene.aurora = None;
            scene
        }
        // A substorm's breakup of soft electrons (red tops), behind broken
        // cloud that hides it where it passes.
        "aurora-breakup" => clouded(
            auroral(Aurora {
                activity: 0.95,
                energy: 0.9,
                intensity: 1.5,
                distance: 60_000.0,
                ..Aurora::default()
            }),
            Clouds {
                coverage: 0.35,
                layer: 0.0,
                ..Clouds::default()
            },
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
