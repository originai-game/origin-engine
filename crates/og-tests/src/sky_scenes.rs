//! The realism wave's sky and weather scenes (R4): the physical sky over
//! the colonnade at noon, at sunset looking into the sun, in twilight after
//! it has set, and on a hazy afternoon; clouds; wet ground with puddles,
//! lying snow and a lightning strike; a canopy keeping the snow and the
//! rain off the ground under it; the aurora on a clear night, and in a
//! substorm behind broken cloud; a moonlit night at full and half moon,
//! and without one.

use crate::atmo_scenes::{colonnade, colonnade_camera};
use crate::kit;
use glam::{Mat4, Quat, Vec2, Vec3};
use og_render::{
    Aurora, Background, Bloom, Camera, Clouds, Environment, Exposure, Fog, PhysicalAtmosphere,
    ProceduralSky, RenderScene, StandardMaterial, SurfaceWeather, day_night, shapes,
};

/// The scenes of this module, in `SCENES` order.
pub const SKY_SCENES: &[&str] = &[
    "sky-physical-noon",
    "sky-physical-sunset",
    "sky-physical-twilight",
    "sky-physical-hazy",
    "sky-physical-moon-full",
    "sky-physical-moon-half",
    "sky-physical-moon-none",
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
    scene.frame.camera = colonnade_camera();
    if into_sun {
        let s = look.sun_direction;
        let bearing = Vec3::new(s.x, 0.0, s.z).normalize_or(Vec3::NEG_Z);
        // Tilted up with the sun (a 15° pitch at most), so a sun up to
        // about 40° high stands in the frame with its glow.
        let lift = (s.y.max(0.0).asin() - 0.05).clamp(0.07, 0.26).tan();
        scene.frame.camera.target =
            scene.frame.camera.position + bearing * 40.0 + Vec3::Y * 40.0 * lift;
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
    physical_look(scene, day_night(hours, 25.0, 70.0).with_atmosphere(&air))
}

/// [`physical_light`] with the sun at `sun` (towards it) over Earth's air:
/// for a shot that aims its sun.
pub fn physical_light_under(scene: &mut RenderScene, sun: Vec3) -> og_render::DayNightLook {
    let look =
        og_render::day_night_under(sun.normalize()).with_atmosphere(&PhysicalAtmosphere::EARTH);
    physical_look(scene, look)
}

fn physical_look(
    scene: &mut RenderScene,
    look: og_render::DayNightLook,
) -> og_render::DayNightLook {
    scene.lighting.light = look.key();
    scene.lighting.environment = Environment {
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
    scene.atmosphere.adaptation = look.adaptation;
    scene.atmosphere.fog = Some(Fog {
        color: look.fog_color,
        sun_color: look.fog_sun.unwrap_or(Fog::default().sun_color),
        // Light ground haze, not homogeneous fog up to the cloud layer:
        // 0.0015/m over 1500m would hide almost 90% even looking straight up.
        density: 0.0,
        height_density: 0.0015,
        height: 0.0,
        falloff: 0.02, // 50m scale height
        sky: 0.0,
        ..Default::default()
    });
    scene.look.post.bloom = Some(Bloom::default());
    look
}

/// `live-sky`: a mirrored orb 2.8 m across sunk to its middle in the
/// colonnade's road at 14:00 under the physical sky, seen from above, under
/// broken rose-tinted cloud lit as the runtime lights a clouded scene;
/// without `clouds` the same layer at no coverage, so the two differ only
/// in the clouds. No probe or screen-space reflection is on: the dome
/// reflects the image-based light alone, and it faces the sky over and
/// behind the camera, out of the frame, so only the live sky capture can
/// bring those clouds, in their tint, into it (its rim shows the capture's
/// ground). 150 s into the wind's drift, where the road stands in a gap of
/// the clouds' shadows (as `clouds-fair`).
pub fn live_sky_scene(clouds: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    colonnade(&mut scene);
    // The air is lit from below by the road the orb lies on (its cobbles'
    // mean colour), not Earth's mean grey: the orb's lower half shows it.
    let air = PhysicalAtmosphere {
        ground_albedo: Vec3::new(0.167, 0.145, 0.102),
        ..PhysicalAtmosphere::EARTH
    };
    physical_light(&mut scene, 14.0, air);
    let sphere = scene.add_mesh(shapes::sphere(64, 32));
    let chrome = scene.add_standard_material(StandardMaterial {
        color: Vec3::splat(0.95),
        metalness: 1.0,
        roughness: 0.02,
        ..Default::default()
    });
    let at = Vec3::new(0.0, 0.0, -1.0);
    scene.add_instance(
        sphere,
        chrome,
        Mat4::from_translation(at) * Mat4::from_scale(Vec3::splat(2.8)),
    );
    scene.frame.camera = colonnade_camera();
    scene.frame.camera.position = Vec3::new(1.6, 3.4, 4.6);
    scene.frame.camera.target = at + Vec3::new(0.0, 1.3, -1.5);
    let cover = Clouds {
        coverage: if clouds { 0.55 } else { 0.0 },
        color: Vec3::new(1.0, 0.6, 0.62),
        layer: 0.0,
        ..Clouds::default()
    };
    cover.light_scene(&mut scene);
    scene.atmosphere.clouds = Some(cover);
    scene.frame.tick = 9000;
    scene
}

/// `scene` under `clouds`, the light dimmed by their cover as the runtime
/// does, the camera tilted up to the sky.
pub(crate) fn clouded(mut scene: RenderScene, clouds: Clouds) -> RenderScene {
    clouds.light_scene(&mut scene);
    scene.atmosphere.clouds = Some(clouds);
    scene = metered(scene);
    scene.frame.camera.target += Vec3::Y * 14.0;
    scene
}

/// `scene` with its camera opened up for the cover over it. A camera
/// opens up under a deck that hides the sun by day, as a photographer's
/// meter does (1.5 stops for a closed overcast at noon, as the deck closes
/// over 0.6 to 0.95 cover): at the clear sky's exposure a noon overcast
/// read as dusk (its sky 0.32 in sRGB).
fn metered(scene: RenderScene) -> RenderScene {
    let cover = scene.atmosphere.clouds.map_or(0.0, |c| c.coverage);
    let t = ((cover - 0.6) / 0.35).clamp(0.0, 1.0);
    if scene.lighting.environment.sky.sun_direction.y > 0.0 {
        exposed(scene, 1.5 * t * t * (3.0 - 2.0 * t))
    } else {
        scene
    }
}

/// The grey overcast over the weathered colonnade.
fn deck() -> Clouds {
    Clouds {
        coverage: 0.9,
        color: Vec3::splat(0.75),
        base: 900.0,
        layer: 0.0,
        ..Clouds::default()
    }
}

/// The weathered colonnade's camera, lowered to look along the ground.
fn along_the_ground(scene: &mut RenderScene) {
    scene.frame.camera.position = Vec3::new(1.5, 1.6, 4.0);
    scene.frame.camera.target = Vec3::new(-1.0, 0.2, -30.0);
}

/// The colonnade under a grey overcast at `hours` with `lying` on its
/// surfaces: what a weather left there (og.WeatherState), nothing falling.
fn weathered(hours: f32, lying: SurfaceWeather) -> RenderScene {
    let mut scene = clouded(
        physical_sky_scene(hours, PhysicalAtmosphere::EARTH, false),
        deck(),
    );
    scene.atmosphere.surface_weather = Some(lying);
    along_the_ground(&mut scene);
    scene
}

/// The same colonnade in the rain on `wind`, `lying` on its surfaces, as
/// og.Weather and og.WeatherState compose: the rain in the air, the closed
/// sky, the dimmed light, the drops on the puddles and the way the wind
/// slants the rain past a roof all come from the one weather
/// ([`crate::atmo_scenes::add_weather`]). Render
/// [`crate::atmo_scenes::WEATHER_FRAMES`] frames so the rain fills the view.
fn raining(hours: f32, lying: SurfaceWeather, wind: Vec3) -> RenderScene {
    let mut scene = physical_sky_scene(hours, PhysicalAtmosphere::EARTH, false);
    scene.atmosphere.clouds = Some(deck());
    scene.atmosphere.surface_weather = Some(lying);
    along_the_ground(&mut scene);
    crate::atmo_scenes::add_weather(&mut scene, og_render::WEATHER_RAIN, 1.0, wind, 18.0);
    metered(scene)
}

/// A market stall: its flat plank roof 8 m wide (x -4 to 4) and 6 m deep
/// (z -9 to -3), its underside 3 m up, on four posts, striped canvas
/// hung from its edges, a counter of produce along its back, crates and
/// barrels by it: in front of the weathered camera, which sees the
/// ground under it.
/// `clustered`: its slab and posts are clustered meshes, drawn through
/// the meshlet path (one level, a cluster per 64 triangles).
pub fn canopy(scene: &mut RenderScene, clustered: bool) {
    use crate::kit;
    let mesh = |scene: &mut RenderScene, data: og_render::MeshData| {
        let clusters = clustered.then(|| one_level(&data));
        let m = scene.add_mesh(data);
        if clusters.is_some() {
            scene.set_mesh_clusters(m, clusters);
        }
        m
    };
    // A plank roof and posts (the kit's planks).
    let wood = kit::surface(scene, "planks");
    let slab = mesh(
        scene,
        kit::block(Vec3::new(-4.0, 3.0, -9.0), Vec3::new(4.0, 3.2, -3.0), 1.5),
    );
    scene.add_instance(slab, wood, Mat4::IDENTITY);
    for x in [-3.8f32, 3.8] {
        for z in [-8.8f32, -3.2] {
            let post = mesh(
                scene,
                kit::block(
                    Vec3::new(x - 0.1, 0.0, z - 0.1),
                    Vec3::new(x + 0.1, 3.0, z + 0.1),
                    1.5,
                ),
            );
            scene.add_instance(post, wood, Mat4::IDENTITY);
        }
    }
    // Striped canvas hung under the roof's front and sides.
    let (front, canvas) = crate::weather_scenes::valance(scene, 8.0, 0.55);
    scene.add_instance(
        front,
        canvas,
        Mat4::from_translation(Vec3::new(0.0, 2.45, -2.98)),
    );
    let (side, canvas) = crate::weather_scenes::valance(scene, 6.0, 0.55);
    for x in [-3.98f32, 3.98] {
        scene.add_instance(
            side,
            canvas,
            Mat4::from_rotation_translation(
                Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                Vec3::new(x, 2.45, -6.0),
            ),
        );
    }
    // The stall's counter of produce along its back, clear of the dry
    // ground before it; crates stacked by its posts and barrels beside.
    let counter = kit::prop(scene, "counter");
    counter.place(scene, Vec3::new(-1.9, 0.0, -8.3), 0.0, 1.35);
    counter.place(scene, Vec3::new(1.9, 0.0, -8.3), 0.08, 1.35);
    let crate_ = kit::prop(scene, "crate");
    let barrel = kit::prop(scene, "barrel");
    for (x, z, y, yaw) in [
        (-4.9f32, -4.0f32, 0.0f32, 0.2f32),
        (-4.95, -4.05, 0.55, 0.5),
        (-5.0, -5.3, 0.0, -0.3),
        (5.0, -8.0, 0.0, 0.4),
    ] {
        crate_.place(scene, Vec3::new(x, y, z), yaw, 0.55);
    }
    barrel.place(scene, Vec3::new(4.9, 0.0, -3.4), 0.0, 1.0);
    barrel.place(scene, Vec3::new(5.4, 0.0, -4.4), 1.2, 1.0);
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
    if let Exposure::Manual { ev: e } = &mut scene.frame.camera.exposure {
        *e += ev;
    }
    scene
}

/// The wet colonnade in the rain at night, 0.03 s after a strike 300 m
/// down the way (`strike`), or the same night without it.
pub fn storm(strike: bool) -> RenderScene {
    let mut scene = raining(
        22.5,
        SurfaceWeather {
            wetness: 1.0,
            puddles: 0.5,
            ..SurfaceWeather::default()
        },
        Vec3::ZERO,
    );
    scene.frame.camera.target = Vec3::new(-1.0, 8.0, -30.0);
    // The storm's own night: a dim blue glow from its deck, so the
    // colonnade reads between two strikes (under the deck alone the night
    // is unlit, and the frames between them were black). A hemisphere
    // light: it adds no reflection, and a strike's flash, which multiplies
    // the sky's light, leaves it as it is.
    scene.lighting.hemisphere = ([0.16, 0.26, 0.55], [0.04, 0.06, 0.10], 4.0);
    if strike {
        // 300 m down the way, over the arch at its end: near enough that
        // the flash's light (its point light hangs over the root with a
        // 600 m reach) falls on the colonnade's faces towards the camera,
        // so the stone reads in the flash, and the bolt stands in the
        // vista between the rows.
        // The bolt is a ribbon in the scene: past the default 1 km far
        // plane it would be clipped.
        scene.frame.camera.far = 4000.0;
        let _ = UNSTRUCK.set(Unstruck {
            environment: scene.lighting.environment.intensity,
            light: scene.lighting.light,
            clouds: scene.atmosphere.clouds.as_ref().map(|c| c.flash),
            lights: scene.lighting.lights.len(),
            ribbons: scene.effects.ribbons.len(),
        });
        og_render::add_lightning(&mut scene, STRIKE_AT, STRIKE_TOP, 3, STRIKE_SECONDS);
    }
    scene
}

/// The storm's strike: 300 m down the way, over the arch at its end, from
/// a cloud base 900 m up, drawn `STRIKE_SECONDS` after it struck.
const STRIKE_AT: Vec3 = Vec3::new(-20.0, 0.0, -300.0);
const STRIKE_TOP: f32 = 900.0;
const STRIKE_SECONDS: f32 = 0.03;
/// Seconds between the storm's strikes after its frame: a flash lasts
/// 0.6 s and the storm's night without one is unlit, so they come close,
/// three in two seconds (a two-second clip loops on the same strike's age).
const STRIKE_EVERY: f32 = 2.0 / 3.0;

/// What a strike changes in the storm, as it stood before its strike.
struct Unstruck {
    environment: f32,
    light: og_render::DirectionalLight,
    clouds: Option<glam::Vec4>,
    lights: usize,
    ribbons: usize,
}

/// The storm before its strike (the same for every build of it).
static UNSTRUCK: std::sync::OnceLock<Unstruck> = std::sync::OnceLock::new();

/// `weather-lightning`'s frames: the rain's ticks, and after the scene's
/// own frames (its strike as drawn, `STRIKE_SECONDS` in) the strike going
/// on a 60th of a second a frame, the double flash and the bolt dying,
/// another strike (the next bolt) every `STRIKE_EVERY` seconds.
pub fn storm_update(scene: &mut RenderScene, frame: u32) {
    crate::fx_scenes::tick_update(scene, frame);
    let settled = crate::atmo_scenes::WEATHER_FRAMES - 1;
    let Some(base) = UNSTRUCK.get().filter(|_| frame > settled) else {
        return;
    };
    scene.lighting.environment.intensity = base.environment;
    scene.lighting.light = base.light;
    if let (Some(c), Some(flash)) = (scene.atmosphere.clouds.as_mut(), base.clouds) {
        c.flash = flash;
    }
    scene.lighting.lights.truncate(base.lights);
    scene.effects.ribbons.truncate(base.ribbons);
    let t = STRIKE_SECONDS + (frame - settled) as f32 / scene.frame.tick_rate.max(1.0);
    let n = (t / STRIKE_EVERY).floor();
    og_render::add_lightning(
        scene,
        STRIKE_AT,
        STRIKE_TOP,
        3 + n as u32,
        t - n * STRIKE_EVERY,
    );
}

/// The colonnade on a clear moonless night under `aurora` (its arcs out
/// along -z, where the camera looks down the road), the camera's target
/// 44 m out raised by `lift` metres (9: 13° up, so the road, the arch at
/// its end and the hills keep the frame's lower third) and opened up three
/// stops for the night. No moon stands in the sky and none lights the
/// land: it lies in the light of the sky as drawn, the aurora's with it.
fn auroral(aurora: Aurora, lift: f32) -> RenderScene {
    let mut scene = exposed(
        physical_sky_scene(23.5, PhysicalAtmosphere::EARTH, false),
        3.0,
    );
    if let Some(look) = scene.lighting.environment.sky.look.as_mut() {
        look.moon_size = 0.0;
    }
    scene.lighting.light.intensity = 0.0;
    scene.atmosphere.aurora = Some(aurora);
    scene.frame.camera.target += Vec3::Y * lift;
    scene
}

/// `sky-physical-moon-*`: grass out to the kit's hills on a clear night
/// under the physical sky, the sun 28° below the horizon and a moon of
/// `size` degrees (0: none) at `phase` 28° up opposite it, a little right
/// of the camera's view along -z, tilted 14° up so the hills lie low in
/// the frame, opened 1.5 stops for the night (more makes a full moon's sky
/// read as evening). A new moon gives no light. The moon lights the land and
/// its light scatters in the air as the sun's does (Jensen et al. 2001):
/// a dim blue sky, brightest round the moon, against the hills' line.
pub fn moon_sky_scene(size: f32, phase: f32) -> RenderScene {
    let mut scene = RenderScene::new();
    let grass = kit::surface_tinted(&mut scene, "grass", [0.86, 1.0, 0.72]);
    let ground = scene.add_mesh(kit::slab(Vec2::splat(-260.0), Vec2::splat(260.0), 0.0, 7.0));
    scene.add_instance(ground, grass, Mat4::IDENTITY);
    kit::hills(&mut scene, None);
    let (elevation, bearing) = (28f32.to_radians(), 12f32.to_radians());
    let moon = Vec3::new(
        bearing.sin() * elevation.cos(),
        elevation.sin(),
        -bearing.cos() * elevation.cos(),
    );
    let look = og_render::day_night_under(-moon)
        .with_moon(size, phase)
        .with_atmosphere(&PhysicalAtmosphere::EARTH);
    physical_look(&mut scene, look);
    let eye = Vec3::new(0.0, 1.7, 0.0);
    scene.frame.camera = Camera {
        position: eye,
        target: eye + Vec3::new(0.0, 40.0 * 14f32.to_radians().tan(), -40.0),
        far: 3000.0,
        exposure: Exposure::linear(0.8),
        ..Default::default()
    };
    exposed(scene, 1.5)
}

/// The scene called `name` of [`SKY_SCENES`].
pub fn sky_scene(name: &str) -> RenderScene {
    let earth = PhysicalAtmosphere::EARTH;
    match name {
        "live-sky" => live_sky_scene(true),
        // live-sky with its clouds cleared (no reference; the live-sky
        // check compares against it).
        "live-sky-clear" => live_sky_scene(false),
        "weather-wet" => raining(
            11.0,
            SurfaceWeather {
                wetness: 1.0,
                puddles: 0.7,
                ..SurfaceWeather::default()
            },
            Vec3::ZERO,
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
            // The rain on a 4 m/s wind from -x, which slants it 20
            // degrees: the dry patch shifts downwind (+x) of the canopy.
            let mut scene = raining(
                11.0,
                SurfaceWeather {
                    wetness: 1.0,
                    puddles: 0.8,
                    ..SurfaceWeather::default()
                },
                Vec3::new(4.0, 0.0, 0.0),
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
        // 150 s into the wind's drift: the colonnade stands in a gap of
        // the clouds' shadows there, sunlit under a half-covered sky (at
        // tick 0 a cloud's shadow lies over the whole of it).
        "clouds-fair" => {
            let mut scene = clouded(
                physical_sky_scene(11.0, earth, false),
                Clouds {
                    coverage: 0.5,
                    ..Clouds::default()
                },
            );
            scene.frame.tick = 9000;
            scene
        }
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
        // Two quiet, faintly rayed arcs, 280 and 480 km out: far enough
        // that a nearly level camera holds the nearer over the colonnade
        // and the farther low over the hills. Opened half a stop more, for
        // a quiet display's light on the land. And their night without
        // them (no reference; the check compares against it).
        "aurora-arcs" => exposed(
            auroral(
                Aurora {
                    activity: 0.3,
                    intensity: 1.6,
                    curtains: 2,
                    distance: 280_000.0,
                    spread: 200_000.0,
                    ..Aurora::default()
                },
                9.0,
            ),
            0.5,
        ),
        "aurora-arcs-clear" => {
            let mut scene = exposed(auroral(Aurora::default(), 9.0), 0.5);
            scene.atmosphere.aurora = None;
            scene
        }
        // A substorm's breakup of soft electrons (red tops), 200 km out:
        // seen from the side its curtains stand as rays over the hills,
        // fold over each other and fill the sky above the colonnade, and
        // their light is on the land.
        "aurora-breakup" => auroral(
            Aurora {
                activity: 0.95,
                energy: 0.9,
                intensity: 1.5,
                distance: 200_000.0,
                ..Aurora::default()
            },
            14.0,
        ),
        "sky-physical-noon" => physical_sky_scene(12.5, earth, false),
        "sky-physical-moon-full" => moon_sky_scene(1.6, 0.5),
        "sky-physical-moon-half" => moon_sky_scene(1.6, 0.25),
        "sky-physical-moon-none" => moon_sky_scene(0.0, 0.0),
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
