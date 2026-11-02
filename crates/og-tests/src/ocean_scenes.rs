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
];

/// The sea's tick (a minute and a half in, at 60 a second).
const TICK: u64 = 5400;

/// A sea of `size` metres a side, 256 cells, under `wind`, around the
/// world's zero (its waves are the world's: the camera at x = z = 0 sees
/// the same ones wherever the origin is).
fn sea(wind: Vec2, size: f32, choppiness: f32) -> Water {
    Water {
        size: Vec2::splat(size),
        segments: 256,
        wind,
        choppiness,
        // Open water: no shore to foam at, nothing below to colour by.
        foam_width: 0.0,
        deep: Vec3::new(0.01, 0.06, 0.09),
        shallow: Vec3::new(0.2, 0.5, 0.55),
        depth_fade: 30.0,
        wave_strength: 0.2,
        wave_scale: 0.9,
        ..Default::default()
    }
}

/// Haze that thickens to the horizon colour at the sea's edge, so its rim
/// (`edge` metres away) does not show against the sky.
fn sea_haze(scene: &mut RenderScene, edge: f32) {
    if let Some(f) = scene.fog.as_mut() {
        f.density = 3.0 / edge;
        f.start = 20.0;
        f.max_opacity = 1.0;
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
            scene.water.push(sea(Vec2::new(-3.0, -8.5), 1200.0, 0.6));
            sea_haze(&mut scene, 600.0);
            scene.camera = Camera {
                position: Vec3::new(0.0, 4.0, 0.0),
                target: Vec3::new(0.0, 1.5, -60.0),
                far: 3000.0,
                exposure: Exposure::linear(0.8),
                ..Default::default()
            };
        }
        // A gale (17 m/s, waves some 230 m long and 6 m high, crests at
        // full choppiness) under an overcast, whitecaps on the crests,
        // seen from a deck 9 m up.
        "water-ocean-storm" => {
            physical_light(&mut scene, 13.0, earth);
            let clouds = Clouds {
                coverage: 0.9,
                density: 0.05,
                color: Vec3::splat(0.75),
                base: 900.0,
                layer: 0.0,
                ..Clouds::default()
            };
            let (sun, sky) = clouds.light();
            scene.light.intensity *= sun;
            scene.environment.intensity *= sky;
            scene.clouds = Some(clouds);
            scene.water.push(sea(Vec2::new(6.0, -16.0), 1600.0, 1.0));
            sea_haze(&mut scene, 800.0);
            scene.camera = Camera {
                position: Vec3::new(0.0, 9.0, 0.0),
                target: Vec3::new(-10.0, 2.0, -80.0),
                far: 4000.0,
                exposure: Exposure::linear(1.0),
                ..Default::default()
            };
        }
        // A light breeze (6 m/s) at sunset, looking into the low sun: the
        // glitter path of the sun's highlight on every wave facing it.
        "water-ocean-sunset" => {
            let look = physical_light(&mut scene, 17.85, earth);
            scene.water.push(sea(Vec2::new(4.0, -4.5), 900.0, 0.5));
            sea_haze(&mut scene, 450.0);
            let s = look.sun_direction;
            let bearing = Vec3::new(s.x, 0.0, s.z).normalize_or(Vec3::NEG_Z);
            scene.camera = Camera {
                position: Vec3::new(0.0, 3.0, 0.0),
                target: Vec3::new(0.0, 3.0, 0.0) + bearing * 60.0 + Vec3::Y * 2.5,
                far: 3000.0,
                exposure: Exposure::linear(1.4),
                ..Default::default()
            };
        }
        "water-flow-map" => flow_map_scene(&mut scene, true),
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

/// A stream 14 m wide in a sandy bed, flowing away from the camera and
/// parting round a boulder, its ripples (the lake normal map of
/// `fork_scenes`) carried by the flow: by the flow map round the boulder
/// (`mapped`), else straight down the stream at the same speed.
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
        color: Vec3::new(0.42, 0.36, 0.26),
        roughness: 0.9,
        ..Default::default()
    });
    scene.add_instance(bed, sand, Mat4::IDENTITY);
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
    let normal_map = crate::fork_scenes::ripple_map(scene);
    scene.water.push(Water {
        transform: Mat4::from_translation(Vec3::new(0.0, 0.9, 0.0)),
        size,
        segments: 32,
        depth_fade: 1.2,
        foam_width: 0.3,
        wave_strength: 1.0,
        normal_map: Some(normal_map),
        normal_scale: 0.5,
        // Down the stream (-z) at 0.25 uv a second; with the map, the speed
        // (the map's half-speed stream turned round the boulder: the same
        // 0.25 away from it).
        flow: Vec2::new(0.0, if mapped { -0.5 } else { -0.25 }),
        flow_map,
        ..Default::default()
    });
    scene.camera = Camera {
        position: Vec3::new(0.0, 6.0, 9.0),
        target: Vec3::new(0.0, 0.9, -8.0),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    // 0.88 s in: the normal map's layers half a cycle along (0.22 of 0.15
    // uv), the travelling waves 0.22 m.
    scene.tick = 53;
    scene.tick_rate = 60.0;
    scene.fog = None::<Fog>;
}
