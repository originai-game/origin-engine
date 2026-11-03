//! The realism wave's water scenes (R5): an open sea under the physical
//! sky (a fresh breeze in the afternoon, a gale with whitecaps under an
//! overcast, a sunset's glitter path), and a stream that a flow map turns
//! round a boulder.

use crate::scenes::texture;
use crate::sky_scenes::physical_light;
use glam::{Mat4, Vec2, Vec3};
use og_render::{
    Camera, Clouds, ColorSpace, Exposure, Fog, PhysicalAtmosphere, RenderScene, StandardMaterial,
    Water, shapes,
};

/// The scenes of this module, in `SCENES` order.
pub const OCEAN_SCENES: &[&str] = &[
    "water-ocean",
    "water-ocean-storm",
    "water-ocean-sunset",
    "water-flow-map",
    "water-ocean-dusk-storm",
    "water-ocean-night",
];

/// The sea's tick (a minute and a half in, at 60 a second).
const TICK: u64 = 5400;

/// A sea 6 km across (its grid finest under the camera, at its centre),
/// 256 cells, under `wind`, around the world's zero (its waves are the
/// world's: the camera at x = z = 0 sees the same ones wherever the origin
/// is). The sea's own waves reach down to under a metre, so the detail
/// ripples are off.
fn sea(wind: Vec2, choppiness: f32) -> Water {
    Water {
        size: Vec2::splat(SEA),
        segments: 256,
        wind,
        choppiness,
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
/// does not show against the sky: gradually, half of it by 1.2 km.
fn sea_haze(scene: &mut RenderScene) {
    if let Some(f) = scene.fog.as_mut() {
        f.density = 3.5 / SEA;
        f.start = 20.0;
        f.max_opacity = 1.0;
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

/// The scene called `name` of [`OCEAN_SCENES`].
pub fn ocean_scene(name: &str) -> RenderScene {
    let earth = PhysicalAtmosphere::EARTH;
    let mut scene = RenderScene::new();
    scene.tick = TICK;
    scene.tick_rate = 60.0;
    match name {
        // A fresh breeze (9 m/s, waves about 65 m long, 1.7 m significant
        // height) running away from the camera and to the left, the
        // afternoon sun behind the right shoulder.
        "water-ocean" => {
            physical_light(&mut scene, 15.0, earth);
            scene.water.push(sea(Vec2::new(-3.0, -8.5), 0.6));
            sea_haze(&mut scene);
            scene.camera = Camera {
                position: Vec3::new(0.0, 4.0, 0.0),
                target: Vec3::new(0.0, 1.5, -60.0),
                far: 5000.0,
                exposure: Exposure::linear(0.8),
                ..Default::default()
            };
        }
        // A strong gale (20 m/s, waves some 330 m long, a significant
        // height near 10 m, crests at full choppiness) under a dark
        // overcast, whitecaps on the crests and old foam streaked along the
        // wind, seen from a deck 10 m up (above every crest).
        "water-ocean-storm" => {
            physical_light(&mut scene, 13.0, earth);
            let clouds = Clouds {
                coverage: 0.97,
                density: 0.09,
                color: Vec3::splat(0.5),
                base: 700.0,
                layer: 0.0,
                ..Clouds::default()
            };
            let (sun, sky) = clouds.light();
            scene.light.intensity *= sun;
            scene.environment.intensity *= sky;
            scene.clouds = Some(clouds);
            let mut gale = sea(Vec2::new(7.0, -18.7), 1.0);
            // Grey-green deep water under the overcast.
            gale.deep = Vec3::new(0.012, 0.035, 0.035);
            scene.water.push(gale);
            sea_haze(&mut scene);
            // 96 s in: the swell's crests under the near sea (at 90 s the
            // view's first 50 m lay in one long trough, with no whitecaps).
            scene.tick = TICK + 360;
            scene.camera = Camera {
                position: Vec3::new(0.0, 10.0, 0.0),
                target: Vec3::new(-10.0, 1.0, -80.0),
                far: 5000.0,
                exposure: Exposure::linear(1.0),
                ..Default::default()
            };
        }
        // A light breeze (6 m/s) at sunset, looking into the low sun: the
        // glitter path of the sun's highlight on every wave facing it.
        "water-ocean-sunset" => {
            let look = physical_light(&mut scene, 17.85, earth);
            scene.water.push(sea(Vec2::new(4.0, -4.5), 0.5));
            sea_haze(&mut scene);
            let s = look.sun_direction;
            let bearing = Vec3::new(s.x, 0.0, s.z).normalize_or(Vec3::NEG_Z);
            scene.camera = Camera {
                position: Vec3::new(0.0, 3.0, 0.0),
                target: Vec3::new(0.0, 3.0, 0.0) + bearing * 60.0 + Vec3::Y * 2.5,
                far: 5000.0,
                exposure: Exposure::linear(1.4),
                ..Default::default()
            };
        }
        "water-flow-map" => flow_map_scene(&mut scene, true),
        // A gale building at dusk (the sun some 5° under the horizon) under a
        // closing deck, seen as the breeze's frame is, metered: the sea as
        // dark as the dim sky it mirrors, its body lit only by the dusk.
        "water-ocean-dusk-storm" => {
            physical_light(&mut scene, 18.4, earth);
            let clouds = Clouds {
                coverage: 0.9,
                density: 0.09,
                color: Vec3::splat(0.5),
                base: 700.0,
                layer: 0.0,
                ..Clouds::default()
            };
            let (sun, sky) = clouds.light();
            scene.light.intensity *= sun;
            scene.environment.intensity *= sky;
            scene.clouds = Some(clouds);
            let mut gale = sea(Vec2::new(5.0, -14.0), 0.85);
            gale.deep = Vec3::new(0.012, 0.035, 0.035);
            scene.water.push(gale);
            sea_haze(&mut scene);
            scene.camera = metered(Vec3::new(0.0, 6.0, 0.0), -1.0);
        }
        // The breeze's sea at night under a clear, starry sky and a moon
        // behind the camera: dark water, the moonlight on it faint.
        "water-ocean-night" => {
            physical_light(&mut scene, 22.3, earth);
            scene.water.push(sea(Vec2::new(-3.0, -8.5), 0.6));
            sea_haze(&mut scene);
            scene.camera = metered(Vec3::new(0.0, 4.0, 0.0), -2.5);
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
    // (water.wgsl `flow_at`: the image's top at +z).
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
    scene.light = og_render::DirectionalLight {
        direction_to_light: sun,
        intensity: 2.5,
        ..Default::default()
    };
    scene.environment = og_render::Environment {
        sky: og_render::ProceduralSky {
            sun_direction: sun,
            ..Default::default()
        },
        intensity: 1.0,
        background: og_render::Background::Sky,
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
    scene.water.push(Water {
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
    scene.camera = Camera {
        position: Vec3::new(0.0, 7.0, 9.0),
        target: Vec3::new(0.0, 0.9, -8.0),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    // 1.5 s in, half a period (water.wgsl `flowed_slope`), when one copy
    // of the waves alone is drawn: moved 0.75 m along the flow far out,
    // 1.5 m beside the boulder and not at all at its nose.
    scene.tick = 90;
    scene.tick_rate = 60.0;
    scene.fog = None::<Fog>;
}
