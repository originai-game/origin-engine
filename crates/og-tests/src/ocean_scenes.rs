//! The realism wave's water scenes (R5): an open sea under the physical
//! sky (a fresh breeze in the afternoon, the same sea in a gale and under
//! the weather's rain, a sunset's glitter path), and a stream that a flow
//! map turns round a boulder.

use crate::scenes::texture;
use crate::sky_scenes::physical_light;
use glam::{Mat4, Vec2, Vec3};
use og_render::{
    Camera, ColorSpace, Exposure, Fog, PhysicalAtmosphere, RenderScene, StandardMaterial, Water,
    shapes,
};

/// The scenes of this module, in `SCENES` order.
pub const OCEAN_SCENES: &[&str] = &[
    "water-ocean",
    "water-ocean-gale",
    "water-ocean-rain",
    "water-ocean-sunset",
    "water-flow-map",
    "water-ocean-night",
    "water-shore",
    "water-shallows",
    "water-wake",
    "water-wake-lake",
    "water-river",
];

/// The sea's tick (a minute and a half in, at 60 a second).
const TICK: u64 = 5400;

/// An endless sea (its clipmap about the camera, out to the horizon)
/// under `wind`, at the world's zero (its waves are the world's: the camera
/// at x = z = 0 sees the same ones wherever the origin is). The sea's own
/// waves reach down to centimetres, so the detail ripples are off.
fn sea(wind: Vec2, choppiness: f32) -> Water {
    Water {
        size: Vec2::splat(SEA),
        segments: 256,
        wind,
        choppiness,
        endless: true,
        // Open water: no shore to foam at, nothing below to colour by.
        foam_width: 0.0,
        deep: Vec3::new(0.01, 0.06, 0.09),
        shallow: Vec3::new(0.2, 0.5, 0.55),
        depth_fade: 30.0,
        wave_strength: 0.0,
        ..Default::default()
    }
}

/// The sea's extent: its edge 3 km out, a pixel under the horizon from a
/// few metres up.
const SEA: f32 = 6000.0;

/// Haze that thickens to the horizon colour by the sea's edge, so its rim
/// does not show against the sky: gradually, half of it by about 1.2 km
/// near sea level. Its density decreases through the marine boundary layer.
fn sea_haze(scene: &mut RenderScene) {
    if let Some(f) = scene.atmosphere.fog.as_mut() {
        // Lenain & Melville 2017, doi:10.1175/JPO-D-16-0058.1, reports
        // marine aerosol layers hundreds of metres deep. A 300m exponential
        // scale is this fixture's approximation, not a measured profile fit.
        f.density = 0.0;
        f.height_density = 3.5 / SEA;
        f.height = 0.0;
        f.falloff = 1.0 / 300.0;
        f.start = 0.0;
        f.max_opacity = 1.0;
    }
}

/// The gale's wind (m/s, xz): 20 m/s, away from the camera and to the right.
const GALE: Vec2 = Vec2::new(7.0, -18.7);

/// The gale's view: the breeze's, from a deck 10 m up.
fn gale_camera() -> Camera {
    Camera {
        position: Vec3::new(0.0, 10.0, 0.0),
        target: Vec3::new(0.0, 7.5, -60.0),
        far: 5000.0,
        exposure: Exposure::linear(0.8),
        ..Default::default()
    }
}

/// A camera at `eye` looking out over the sea as the breeze's frame does,
/// metered from the frame (an exposure for the view) less `compensation` EV,
/// as a game's camera at dusk or night is.
fn metered(eye: Vec3, compensation: f32) -> Camera {
    Camera {
        position: eye,
        target: Vec3::new(0.0, eye.y - 2.5, -60.0),
        far: 5000.0,
        exposure: Exposure::Auto {
            compensation,
            min_ev: -10.0,
            max_ev: 10.0,
            adapt_speed_up: 0.0,
            adapt_speed_down: 0.0,
        },
        ..Default::default()
    }
}

/// `water-wake`'s boat: its half extents (m; a 2.4 by 6 m hull, 1 m
/// deep), its draft, its speed along -z (m/s) and the frames it runs (one
/// tick each: 5 s).
pub const WAKE_HALF: Vec3 = Vec3::new(1.2, 0.5, 3.0);
pub const WAKE_DRAFT: f32 = 0.5;
pub const WAKE_SPEED: f32 = 5.0;
pub const WAKE_FRAMES: u32 = 300;

/// Where `water-wake`'s boat is `frame` ticks in (its centre, frame xz):
/// from z = 10 towards -z at [`WAKE_SPEED`].
pub fn wake_boat(frame: u32) -> Vec2 {
    Vec2::new(0.0, 10.0 - WAKE_SPEED * frame as f32 / 60.0)
}

/// The boat's footprint `frame` ticks in, as og.Buoyancy's step records it.
fn wake_cut(frame: u32) -> og_render::WaterCut {
    let centre = wake_boat(frame);
    let half = Vec2::new(WAKE_HALF.x, WAKE_HALF.z);
    og_render::WaterCut {
        entity: 1,
        min: centre - half,
        max: centre + half,
        centre,
        half,
        axis: Vec2::X,
        depth: WAKE_DRAFT,
        speed: 0.0,
    }
}

/// `water-wake`'s update: the tick, the boat's hull and the cuts of the
/// last two ticks.
pub fn wake_update(scene: &mut RenderScene, frame: u32) {
    scene.frame.tick = TICK + u64::from(frame);
    let at = wake_boat(frame);
    if let Some(boat) = scene.objects.instances.first_mut() {
        boat.transform = Mat4::from_translation(Vec3::new(at.x, WAKE_HALF.y - WAKE_DRAFT, at.y));
    }
    let ticks = (frame.saturating_sub(1)..=frame)
        .map(|f| (TICK + u64::from(f), vec![wake_cut(f)]))
        .collect();
    if let Some(w) = scene.world.water.first_mut() {
        w.wakes = Some(std::sync::Arc::new(og_render::WaterWakes {
            epoch: 1,
            dt: 1.0 / 60.0,
            ticks,
            ..Default::default()
        }));
    }
}

/// A boat running at 5 m/s over a light breeze's sea (`water-wake`, before
/// [`wake_update`]): its bow wave, its Kelvin wake and the foam along its
/// sides, seen from behind and above. On a `lake` (`water-wake-lake`), a
/// windless water 400 m across, the same wake bends the flat surface's
/// reflections: any og.Water draws the bodies that cut it.
fn wake(scene: &mut RenderScene, lake: bool) {
    physical_light(scene, 14.0, PhysicalAtmosphere::EARTH);
    let hull = scene.add_mesh(shapes::cuboid(WAKE_HALF * 2.0));
    let paint = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.55, 0.12, 0.08),
        roughness: 0.6,
        ..Default::default()
    });
    scene.add_instance(hull, paint, Mat4::IDENTITY);
    if lake {
        scene.world.water.push(Water {
            size: Vec2::splat(400.0),
            deep: Vec3::new(0.01, 0.06, 0.09),
            shallow: Vec3::new(0.2, 0.5, 0.55),
            depth_fade: 30.0,
            ..Default::default()
        });
    } else {
        scene.world.water.push(sea(Vec2::new(-1.2, -2.6), 0.4));
        sea_haze(scene);
    }
    scene.frame.camera = Camera {
        position: Vec3::new(9.0, 16.0, 22.0),
        target: Vec3::new(0.0, 0.0, -6.0),
        far: 5000.0,
        exposure: Exposure::linear(0.8),
        ..Default::default()
    };
    wake_update(scene, 0);
}

/// `water-river`'s course: along a terrace 4 m up, over its edge in a
/// 4 m fall, then on across the plain below (frame xz and the water's level).
pub const RIVER_COURSE: [Vec3; 6] = [
    Vec3::new(-40.0, 4.0, -38.0),
    Vec3::new(-18.0, 4.0, -20.0),
    Vec3::new(-0.5, 4.0, -8.0),
    Vec3::new(0.5, 0.0, -7.5),
    Vec3::new(14.0, 0.0, 4.0),
    Vec3::new(40.0, 0.0, 16.0),
];
pub const RIVER_WIDTH: f32 = 7.0;
pub const RIVER_SPEED: f32 = 1.8;

/// The ground of `water-river`'s valley at frame point `p` (xz): the
/// river's bed 1.2 m under its level mid-stream, its banks rising to half a
/// metre over it a couple of metres past its edge, and the level of the
/// course's nearest point beyond (the terrace and the plain, a cliff where
/// the river falls).
fn river_ground(course: &og_core::river::River, p: Vec2) -> f32 {
    let Some(at) = course.at(p) else {
        return -1.0;
    };
    let s = at.across.abs() * 100.0;
    let half = RIVER_WIDTH * 0.5;
    let bank = (s - (half - 1.5)) / 3.5;
    let t = bank.clamp(0.0, 1.0);
    let rise = t * t * (3.0 - 2.0 * t);
    let hills =
        0.4 * ((p.x * 0.11).sin() * (p.y * 0.07).cos()) * ((s - half - 3.0) / 10.0).clamp(0.0, 1.0);
    at.level - 1.2 + 1.7 * rise + hills
}

/// A river down a terrace into a waterfall and on across the plain
/// (`water-river`): its current turning with its course, slowing to its
/// banks, the fall's sheet and the white water of its pool.
fn river(scene: &mut RenderScene) {
    // Mid-morning: the sun in the east, on the fall's face (it falls east).
    physical_light(scene, 10.0, PhysicalAtmosphere::EARTH);
    let course =
        og_core::river::River::new(&RIVER_COURSE, RIVER_WIDTH, RIVER_SPEED).expect("a river");
    // The valley: a grid 120 m square at half a metre, its heights the ground's.
    // (The course run on 100 m past either end, so the ground there follows its nearest level too.)
    let mut long = RIVER_COURSE.to_vec();
    let (a, b) = (RIVER_COURSE[0], RIVER_COURSE[1]);
    long.insert(
        0,
        a + (a - b).normalize() * 100.0 * Vec3::new(1.0, 0.0, 1.0),
    );
    let (y, z) = (
        RIVER_COURSE[RIVER_COURSE.len() - 1],
        RIVER_COURSE[RIVER_COURSE.len() - 2],
    );
    long.push(y + (y - z).normalize() * 100.0 * Vec3::new(1.0, 0.0, 1.0));
    let wide = og_core::river::River::new(&long, 200.0, 0.0).expect("a course");
    let (n, size) = (241usize, 120.0f32);
    let cell = size / (n - 1) as f32;
    let height = |i: usize, j: usize| {
        river_ground(
            &wide,
            Vec2::new(-size * 0.5 + i as f32 * cell, -size * 0.5 + j as f32 * cell),
        )
    };
    let mut vertices = Vec::with_capacity(n * n);
    for j in 0..n {
        for i in 0..n {
            let (l, r) = (
                height(i.saturating_sub(1), j),
                height((i + 1).min(n - 1), j),
            );
            let (d, u) = (
                height(i, j.saturating_sub(1)),
                height(i, (j + 1).min(n - 1)),
            );
            let normal = Vec3::new(l - r, 2.0 * cell, d - u).normalize();
            vertices.push(og_render::Vertex {
                position: [
                    -size * 0.5 + i as f32 * cell,
                    height(i, j),
                    -size * 0.5 + j as f32 * cell,
                ],
                normal: normal.to_array(),
                uv: [i as f32 * cell * 0.25, j as f32 * cell * 0.25],
                color: [1.0; 4],
                tangent: [0.0; 4],
            });
        }
    }
    let mut indices = Vec::with_capacity(6 * (n - 1) * (n - 1));
    for j in 0..(n - 1) as u32 {
        for i in 0..(n - 1) as u32 {
            let a = j * n as u32 + i;
            let (b, c, d) = (a + 1, a + n as u32, a + n as u32 + 1);
            indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }
    let ground = scene.add_mesh(og_render::MeshData { vertices, indices });
    let earth = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.26, 0.3, 0.16),
        roughness: 0.92,
        ..Default::default()
    });
    scene.add_instance(ground, earth, Mat4::IDENTITY);
    // A clear upland river: green-brown over its bed, dark where deep.
    scene.world.water.push(Water {
        size: Vec2::splat(size),
        shallow: Vec3::new(0.22, 0.32, 0.24),
        deep: Vec3::new(0.03, 0.08, 0.07),
        foam_width: 0.6,
        depth_fade: 2.5,
        river: Some(std::sync::Arc::new(course)),
        ..Default::default()
    });
    // From the plain below the fall, high enough to see the reach above it.
    scene.frame.camera = Camera {
        position: Vec3::new(18.0, 10.0, 14.0),
        target: Vec3::new(-6.0, 2.5, -12.0),
        far: 2000.0,
        exposure: Exposure::linear(0.8),
        ..Default::default()
    };
}

/// Clear sea water over a white slope (`water-shallows`): the seabed's
/// depth under the surface at frame x (m), from 0 at x = -[`SHALLOWS_HALF`]
/// to 20 at +[`SHALLOWS_HALF`].
pub fn shallows_depth(x: f32) -> f32 {
    ((x + SHALLOWS_HALF) / (2.0 * SHALLOWS_HALF) * 20.0).max(0.0)
}

/// Half the width of `water-shallows`' slope across the frame (m).
pub const SHALLOWS_HALF: f32 = 12.0;

fn shallows(scene: &mut RenderScene) {
    let slope = (20.0f32 / (2.0 * SHALLOWS_HALF)).atan();
    let bed = scene.add_mesh(shapes::plane(200.0, 1.0));
    let white = scene.add_standard_material(StandardMaterial {
        color: Vec3::splat(0.8),
        roughness: 0.95,
        ..Default::default()
    });
    // Tilted about z, down to the right; its height at x = 0 is -10 m.
    scene.add_instance(
        bed,
        white,
        Mat4::from_translation(Vec3::new(0.0, -10.0, 0.0)) * Mat4::from_rotation_z(-slope),
    );
    // A light breeze's sea (4 m/s) over it, not endless: its waves focus
    // the sun into caustics on the slope (the waves' own, og.Water.caustics).
    scene.world.water.push(Water {
        size: Vec2::splat(200.0),
        segments: 256,
        wind: Vec2::new(1.5, -3.7),
        choppiness: 0.5,
        caustics: 1.0,
        foam_width: 0.0,
        wave_strength: 0.0,
        // Clear ocean water: pure water's absorption (Pope & Fry 1997 at
        // about 650, 550 and 450 nm) and a little scattering by particles.
        absorption: Vec3::new(0.34, 0.057, 0.0092),
        scattering: Vec3::new(0.01, 0.015, 0.025),
        ..Default::default()
    });
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 22.0, 0.5),
        target: Vec3::ZERO,
        far: 1000.0,
        exposure: Exposure::linear(0.8),
        ..Default::default()
    };
}

/// The scene called `name` of [`OCEAN_SCENES`].
pub fn ocean_scene(name: &str) -> RenderScene {
    let earth = PhysicalAtmosphere::EARTH;
    let mut scene = RenderScene::new();
    scene.frame.tick = TICK;
    scene.frame.tick_rate = 60.0;
    match name {
        // A fresh breeze (9 m/s, waves about 65 m long, 1.7 m significant
        // height) running away from the camera and to the left, the
        // afternoon sun behind the right shoulder.
        "water-ocean" => {
            physical_light(&mut scene, 15.0, earth);
            scene.world.water.push(sea(Vec2::new(-3.0, -8.5), 0.6));
            sea_haze(&mut scene);
            scene.frame.camera = Camera {
                position: Vec3::new(0.0, 4.0, 0.0),
                target: Vec3::new(0.0, 1.5, -60.0),
                far: 5000.0,
                exposure: Exposure::linear(0.8),
                ..Default::default()
            };
        }
        // The same sea, afternoon and settings in a gale (20 m/s: waves
        // some 330 m long, a significant height near 9 m), from 10 m up,
        // above every crest. Only the wind differs from `water-ocean`.
        "water-ocean-gale" => {
            physical_light(&mut scene, 15.0, earth);
            scene.world.water.push(sea(GALE, 0.6));
            sea_haze(&mut scene);
            scene.frame.camera = gale_camera();
        }
        // The gale under the world's rain on the same wind, as a game's
        // og.Wind, og.Weather and og.Water compose: the weather closes the
        // sky and dims the light, and the sea takes both from the frame
        // (nothing on the water is set for it).
        "water-ocean-rain" => {
            physical_light(&mut scene, 15.0, earth);
            scene.world.water.push(sea(GALE, 0.6));
            sea_haze(&mut scene);
            scene.frame.camera = gale_camera();
            crate::atmo_scenes::add_weather(
                &mut scene,
                og_render::WEATHER_RAIN,
                1.5,
                Vec3::new(GALE.x, 0.0, GALE.y),
                40.0,
            );
        }
        // A light breeze (6 m/s) at sunset, looking into the low sun: the
        // glitter path of the sun's highlight on every wave facing it.
        "water-ocean-sunset" => {
            let look = physical_light(&mut scene, 17.85, earth);
            scene.world.water.push(sea(Vec2::new(4.0, -4.5), 0.5));
            sea_haze(&mut scene);
            let s = look.sun_direction;
            let bearing = Vec3::new(s.x, 0.0, s.z).normalize_or(Vec3::NEG_Z);
            scene.frame.camera = Camera {
                position: Vec3::new(0.0, 3.0, 0.0),
                target: Vec3::new(0.0, 3.0, 0.0) + bearing * 60.0 + Vec3::Y * 2.5,
                far: 5000.0,
                exposure: Exposure::linear(1.4),
                ..Default::default()
            };
        }
        "water-flow-map" => flow_map_scene(&mut scene, true),
        // W2: clear sea water (og.Water.absorption and scattering) over a
        // white seabed sloping from the waterline on the left to 20 m deep on
        // the right, seen from 22 m above in the late morning: the light
        // through it goes from the sand's own to turquoise to deep blue as
        // the red, then the green, is absorbed on the way down and back,
        // and the waves' caustics play over the shallows.
        "water-shallows" => {
            physical_light(&mut scene, 10.5, earth);
            shallows(&mut scene);
        }
        "water-wake" => wake(&mut scene, false),
        "water-wake-lake" => wake(&mut scene, true),
        "water-river" => river(&mut scene),
        // The breeze's sea at night under a clear, starry sky and a moon
        // behind the camera: dark water, the moonlight on it faint.
        "water-ocean-night" => {
            physical_light(&mut scene, 22.3, earth);
            scene.world.water.push(sea(Vec2::new(-3.0, -8.5), 0.6));
            sea_haze(&mut scene);
            scene.frame.camera = metered(Vec3::new(0.0, 4.0, 0.0), -2.5);
        }
        // W1: a breeze's sea rolling in on a 1:20 sandy beach, seen from
        // the water 160 m out: the swell fading in the shallows, the shore
        // trains breaking parallel to the beach into surf.
        "water-shore" => {
            physical_light(&mut scene, 15.0, earth);
            beach(&mut scene, sea(Vec2::new(1.5, -8.0), 0.6), 1.0);
            sea_haze(&mut scene);
            scene.frame.camera = Camera {
                position: Vec3::new(0.0, 6.0, 60.0),
                target: Vec3::new(0.0, 0.0, -100.0),
                far: 5000.0,
                exposure: Exposure::linear(0.8),
                ..Default::default()
            };
        }
        other => panic!("no ocean scene {other}"),
    }
    scene
}

/// The flow map's texel at (`u`, `v`) of a stream of `size` round a
/// boulder of `radius` metres at `at` (the stream's xz): the potential flow
/// of a half-speed stream along -z past a cylinder, ∇(U x_D (1 + R²/r²)) =
/// U (D (1 + R²/r²) - 2 R² (D·r) r / r⁴) (twice the speed at its sides,
/// none at its nose and its tail; none inside it), as rg `* 0.5 + 0.5` (u
/// along x, v along z from the -x -z corner).
fn flow_round(size: Vec2, at: Vec2, radius: f32, u: f32, v: f32) -> [u8; 4] {
    // The texture's rows run top to bottom; the stream's v runs up the image
    // (water_detail.wgsl `flow_at`: the image's top at +z).
    let r = Vec2::new((u - 0.5) * size.x, (0.5 - v) * size.y) - at;
    let (d, speed) = (Vec2::new(0.0, -1.0), 0.5);
    let (r2, k) = (r.length_squared(), radius * radius);
    let f = if r2 <= k {
        Vec2::ZERO
    } else {
        (d * (1.0 + k / r2) - r * (2.0 * k * d.dot(r) / (r2 * r2))) * speed
    };
    let e = |c: f32| ((c.clamp(-1.0, 1.0) * 0.5 + 0.5) * 255.0).round() as u8;
    [e(f.x), e(f.y), 128, 255]
}

/// A stream 14 m wide between two banks, flowing away from the camera and
/// parting round a boulder, its travelling waves carried by the flow: by
/// the flow map round the boulder (`mapped`; half a metre a second far from
/// it, a metre beside it, none at its nose), else straight down the stream
/// at the far field's speed. (A flow map moves the waves by up to a metre
/// in the two seconds of a cycle, so in a still their fronts bend round the
/// boulder, the fronts lying across the stream.)
pub fn flow_map_scene(scene: &mut RenderScene, mapped: bool) {
    let sun = Vec3::new(0.35, 0.75, 0.45).normalize();
    scene.lighting.light = og_render::DirectionalLight {
        direction_to_light: sun,
        intensity: 2.5,
        ..Default::default()
    };
    scene.lighting.environment = og_render::Environment {
        sky: og_render::ProceduralSky {
            sun_direction: sun,
            ..Default::default()
        },
        intensity: 1.0,
        background: og_render::Background::Sky,
        ..Default::default()
    };
    let bed = scene.add_mesh(shapes::plane(80.0, 1.0));
    let sand = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.62, 0.52, 0.36),
        roughness: 0.9,
        ..Default::default()
    });
    scene.add_instance(bed, sand, Mat4::IDENTITY);
    // The banks: grassy slabs 1.5 m high either side of the stream.
    let slab = scene.add_mesh(shapes::cuboid(Vec3::ONE));
    let grass = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.3, 0.42, 0.2),
        roughness: 0.95,
        ..Default::default()
    });
    for x in [-1.0f32, 1.0] {
        scene.add_instance(
            slab,
            grass,
            Mat4::from_scale_rotation_translation(
                Vec3::new(10.0, 1.5, 80.0),
                glam::Quat::IDENTITY,
                Vec3::new(x * 12.0, 0.75, -20.0),
            ),
        );
    }
    let rock = scene.add_mesh(shapes::sphere(32, 16));
    let stone = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.32, 0.31, 0.3),
        roughness: 0.8,
        ..Default::default()
    });
    let at = Vec2::new(0.0, -6.0);
    let radius = 1.4;
    scene.add_instance(
        rock,
        stone,
        Mat4::from_scale_rotation_translation(
            Vec3::new(radius, 1.2, radius),
            glam::Quat::IDENTITY,
            Vec3::new(at.x, 0.6, at.y),
        ),
    );
    let size = Vec2::new(14.0, 40.0);
    let flow_map = mapped.then(|| {
        scene.add_texture(texture(128, ColorSpace::Linear, |u, v| {
            flow_round(size, at, radius, u, v)
        }))
    });
    scene.world.water.push(Water {
        transform: Mat4::from_translation(Vec3::new(0.0, 0.9, 0.0)),
        size,
        segments: 32,
        depth_fade: 1.2,
        foam_width: 0.3,
        wave_scale: 1.0,
        wave_strength: 0.6,
        // With the map, its half-speed stream times 1 m/s; without, the
        // same half a metre a second straight down the stream (-z).
        flow: Vec2::new(0.0, if mapped { -1.0 } else { -0.5 }),
        flow_map,
        ..Default::default()
    });
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 7.0, 9.0),
        target: Vec3::new(0.0, 0.9, -8.0),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    // 1.5 s in, half a period (water_detail.wgsl `flowed_slope`), when one copy
    // of the waves alone is drawn: moved 0.75 m along the flow far out,
    // 1.5 m beside the boulder and not at all at its nose.
    scene.frame.tick = 90;
    scene.frame.tick_rate = 60.0;
    scene.atmosphere.fog = None::<Fog>;
}

/// The beach's extent (m) and samples a side (2 m apart).
const BEACH: f32 = 600.0;
const BEACH_SAMPLES: u32 = 301;
/// Where the waterline crosses x = 0 (z, m).
pub const BEACH_LINE: f32 = -100.0;

/// The beach's height at (x, z): a 1:20 slope rising towards -z, its
/// waterline at z = [`BEACH_LINE`] bowed by a cove of 15 m along x, the sea
/// floor levelling at 30 m down.
pub fn beach_height(x: f32, z: f32) -> f32 {
    let cove = 15.0 * (x / 140.0).cos();
    ((BEACH_LINE + cove - z) / 20.0).max(-30.0)
}

/// The beach's seabed for a sea at y = 0, as og.Water.seabed builds it
/// (og-runtime seabed.rs: the terrain's heights on its own grid).
pub fn beach_shore() -> og_core::ocean::Shore {
    let n = BEACH_SAMPLES;
    let cell = BEACH / (n - 1) as f32;
    let depth: Vec<f32> = (0..n * n)
        .map(|i| {
            let (x, z) = (
                -BEACH * 0.5 + (i % n) as f32 * cell,
                -BEACH * 0.5 + (i / n) as f32 * cell,
            );
            -beach_height(x, z)
        })
        .collect();
    og_core::ocean::Shore::new(Vec2::splat(-BEACH * 0.5), cell, n, n, &depth).expect("a shore")
}

/// The beach in `scene` (its terrain, sand and rock by slope) under the sea
/// `water`, its seabed the beach's and its trains `strength` high.
fn beach(scene: &mut RenderScene, mut water: Water, strength: f32) {
    use og_render::{Terrain, TerrainControl, TerrainData};
    use std::sync::Arc;
    let n = BEACH_SAMPLES;
    let cell = BEACH / (n - 1) as f32;
    let heights: Vec<f32> = (0..n * n)
        .map(|i| {
            beach_height(
                -BEACH * 0.5 + (i % n) as f32 * cell,
                -BEACH * 0.5 + (i / n) as f32 * cell,
            )
        })
        .collect();
    let handle = scene.add_terrain(TerrainData {
        size: Vec2::splat(BEACH),
        columns: n,
        rows: n,
        heights: Arc::from(heights),
        // Sand (blue) with rock (green) left to its slope rule.
        control: Some(TerrainControl {
            width: 2,
            height: 2,
            rgba: Arc::from([0u8, 255, 255, 0].repeat(4)),
        }),
        holes: None,
    });
    let layers = crate::terrain_scenes::island_layers(scene);
    scene.world.terrains.push(Terrain {
        layers,
        height_blend: 0.6,
        ..Terrain::new(handle)
    });
    let field = std::sync::Arc::new(beach_shore());
    let sea = og_core::ocean::Ocean::new(water.wind, water.swell, water.choppiness).expect("a sea");
    water.shore = Some(og_render::WaterShore {
        origin: field.origin,
        trains: og_core::ocean::ShoreTrains::new(sea.significant_height(), strength, 0.0),
        field,
        foam: 1.0,
    });
    // A shore to foam at, and a bed to colour by.
    water.foam_width = 0.35;
    water.depth_fade = 6.0;
    scene.world.water.push(water);
}
