//! Weather on surfaces and cloth (Wave B): a rain-wet street at dusk with
//! its lamps lit, puddles drying after the rain, snow built up on ledges,
//! crates and rocks, a dusty desert yard, and velvet and cotton drapes
//! under the sky. Each frame is drawn from a [`SurfaceWeather`] the way the
//! runtime fills it from og.WeatherState.

use crate::sky_scenes::physical_light;
use glam::{Mat4, Quat, Vec3};
use og_render::{
    Camera, Clouds, ColorSpace, Exposure, Light, LightKind, MeshData, Mips, PhysicalAtmosphere,
    RenderScene, ScreenSpaceReflections, StandardMaterial, SurfaceWeather, TextureData, Vertex,
    shapes,
};

/// The scenes of this module, in `SCENES` order.
pub const WEATHER_SCENES: &[&str] = &[
    "weather-street-wet",
    "weather-puddles",
    "weather-snow-buildup",
    "weather-dust-desert",
    "materials-cloth",
    "weather-cobbles-wet",
];

/// Smooth value noise over the plane (0..1), wrapping every `period` cells.
fn value_noise(x: f32, y: f32, period: i32, seed: u32) -> f32 {
    let hash = |i: i32, j: i32| {
        let (i, j) = (i.rem_euclid(period) as u32, j.rem_euclid(period) as u32);
        let mut h = i.wrapping_mul(0x8da6_b343)
            ^ j.wrapping_mul(0xd816_3841)
            ^ seed.wrapping_mul(0xcb1a_b31f);
        h ^= h >> 13;
        h = h.wrapping_mul(0x5bd1_e995);
        h ^= h >> 15;
        (h & 0xffff) as f32 / 65535.0
    };
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - x.floor(), y - y.floor());
    let (u, v) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let a = hash(ix, iy) + (hash(ix + 1, iy) - hash(ix, iy)) * u;
    let b = hash(ix, iy + 1) + (hash(ix + 1, iy + 1) - hash(ix, iy + 1)) * u;
    a + (b - a) * v
}

/// Three octaves of [`value_noise`], tileable over the unit square.
fn fbm(u: f32, v: f32, cells: i32, seed: u32) -> f32 {
    let mut sum = 0.0;
    let mut amp = 0.5;
    let mut c = cells;
    for k in 0..3 {
        sum += amp * value_noise(u * c as f32, v * c as f32, c, seed + k);
        amp *= 0.5;
        c *= 2;
    }
    sum / 0.875
}

fn texture(size: u32, color_space: ColorSpace, f: impl Fn(f32, f32) -> [f32; 3]) -> TextureData {
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let c = f(
                (x as f32 + 0.5) / size as f32,
                (y as f32 + 0.5) / size as f32,
            );
            rgba.extend(c.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8));
            rgba.push(255);
        }
    }
    TextureData {
        width: size,
        height: size,
        rgba,
        color_space,
        mips: Mips::Generate,
    }
}

/// A ground's relief as an AO map (red: its crevices dark): the weather
/// fills the low parts first. Broad dips (a worn road's ruts and
/// hollows) and a fine grain.
fn relief(seed: u32) -> TextureData {
    texture(256, ColorSpace::Linear, |u, v| {
        let h = 0.7 * fbm(u, v, 4, seed) + 0.3 * fbm(u, v, 32, seed + 9);
        [h; 3]
    })
}

/// A ground colour: `base` broken up by blotches and grain (sRGB).
fn mottled(base: [f32; 3], seed: u32) -> TextureData {
    texture(256, ColorSpace::Srgb, |u, v| {
        let t = 0.75 + 0.35 * fbm(u, v, 8, seed) + 0.15 * (fbm(u, v, 64, seed + 5) - 0.5);
        base.map(|c| c * t)
    })
}

fn put(scene: &mut RenderScene, mesh: og_render::MeshHandle, m: StandardMaterial, t: Mat4) {
    let h = scene.add_standard_material(m);
    scene.add_instance(mesh, h, t);
}

/// A flat ground `size` metres across with its colour and relief tiled
/// `repeat` times.
fn ground(scene: &mut RenderScene, size: f32, repeat: f32, m: StandardMaterial) {
    let mesh = scene.add_mesh(shapes::plane(size, repeat));
    put(scene, mesh, m, Mat4::IDENTITY);
}

fn bx(scene: &mut RenderScene, size: Vec3, at: Vec3, m: StandardMaterial) {
    let mesh = scene.add_mesh(shapes::cuboid(size));
    put(scene, mesh, m, Mat4::from_translation(at));
}

/// The scene called `name` of [`WEATHER_SCENES`].
pub fn weather_scene(name: &str) -> RenderScene {
    match name {
        "weather-street-wet" => street(),
        "weather-puddles" => puddles(),
        "weather-snow-buildup" => snow_buildup(),
        "weather-dust-desert" => desert(),
        "materials-cloth" => cloth(),
        "weather-cobbles-wet" => cobbles_wet(),
        _ => panic!("no weather scene {name}"),
    }
}

/// An overcast physical sky at `hours`.
fn overcast(scene: &mut RenderScene, hours: f32, coverage: f32) {
    physical_light(scene, hours, PhysicalAtmosphere::EARTH);
    scene.clouds = Some(Clouds {
        coverage,
        color: Vec3::splat(0.7),
        base: 900.0,
        layer: 0.0,
        ..Clouds::default()
    });
}

/// A street at dusk in the rain: asphalt between two kerbed pavements and
/// brick walls, a parked lacquered box, lamps lit along it. The road is
/// soaked and dark, its ruts hold rippling puddles that mirror the lamps,
/// the rain runs down the walls in streaks, and the lacquer beads.
fn street() -> RenderScene {
    let mut scene = RenderScene::new();
    overcast(&mut scene, 18.5, 0.8);
    let asphalt = StandardMaterial {
        color: Vec3::splat(0.6),
        map: scene.add_texture(mottled([0.17, 0.17, 0.18], 3)).slot(),
        aoMap: scene.add_texture(relief(11)).slot(),
        aoMapIntensity: 0.6,
        roughness: 0.85,
        ..Default::default()
    };
    ground(&mut scene, 80.0, 10.0, asphalt);
    let concrete = StandardMaterial {
        color: Vec3::new(0.42, 0.41, 0.39),
        map: scene.add_texture(mottled([0.9, 0.9, 0.88], 7)).slot(),
        roughness: 0.92,
        ..Default::default()
    };
    let brick = StandardMaterial {
        color: Vec3::new(0.36, 0.17, 0.12),
        map: scene.add_texture(mottled([0.95, 0.85, 0.8], 19)).slot(),
        roughness: 0.9,
        ..Default::default()
    };
    for side in [-1.0f32, 1.0] {
        // The pavement, its kerb 15 cm up, and the wall behind it.
        bx(
            &mut scene,
            Vec3::new(3.0, 0.15, 80.0),
            Vec3::new(side * 5.5, 0.075, -20.0),
            concrete,
        );
        bx(
            &mut scene,
            Vec3::new(1.0, 9.0, 80.0),
            Vec3::new(side * 7.5, 4.5, -20.0),
            brick,
        );
        for k in 0..4 {
            let z = -4.0 - 12.0 * k as f32;
            // A lamp post on the kerb and its warm lamp.
            let metal = StandardMaterial {
                color: Vec3::splat(0.08),
                metalness: 1.0,
                roughness: 0.4,
                ..Default::default()
            };
            bx(
                &mut scene,
                Vec3::new(0.12, 4.2, 0.12),
                Vec3::new(side * 4.3, 2.25, z),
                metal,
            );
            bx(
                &mut scene,
                Vec3::new(0.35, 0.15, 0.35),
                Vec3::new(side * 4.0, 4.4, z),
                StandardMaterial {
                    color: Vec3::ZERO,
                    emissive: Vec3::new(1.0, 0.7, 0.4),
                    emissiveIntensity: 8.0,
                    weathering: 0.0,
                    ..Default::default()
                },
            );
            scene.lights.push(Light {
                kind: LightKind::Point,
                position: Vec3::new(side * 4.0, 4.2, z),
                color: Vec3::new(1.0, 0.72, 0.45),
                intensity: 40.0,
                range: 18.0,
                cast_shadows: false,
            });
        }
    }
    // A lacquered box parked by the kerb: sealed paint, no pores to fill.
    bx(
        &mut scene,
        Vec3::new(1.8, 1.2, 4.2),
        Vec3::new(2.8, 0.75, -11.0),
        StandardMaterial {
            color: Vec3::new(0.45, 0.03, 0.03),
            roughness: 0.35,
            clearcoat: 1.0,
            clearcoatRoughness: 0.05,
            ..Default::default()
        },
    );
    scene.surface_weather = Some(SurfaceWeather {
        wetness: 1.0,
        puddles: 0.75,
        rain: 1.0,
        puddle_size: 2.5,
        ..SurfaceWeather::default()
    });
    // The puddles and the lacquer mirror the lamps and the walls.
    scene.look.post.ssr = Some(ScreenSpaceReflections::default());
    scene.camera = Camera {
        position: Vec3::new(-1.2, 1.5, 6.0),
        target: Vec3::new(0.6, 0.9, -30.0),
        exposure: Exposure::Manual { ev: 0.5 },
        ..Default::default()
    };
    scene
}

/// Rocky ground under a clearing sky half an hour after the rain: the
/// puddles stand still and mirror the sky, and the ground dries in
/// patches from its high spots, the hollows and the shores still dark.
fn puddles() -> RenderScene {
    let mut scene = RenderScene::new();
    overcast(&mut scene, 15.0, 0.55);
    let dirt = StandardMaterial {
        color: Vec3::splat(0.85),
        map: scene.add_texture(mottled([0.42, 0.34, 0.26], 23)).slot(),
        aoMap: scene.add_texture(relief(29)).slot(),
        aoMapIntensity: 0.5,
        roughness: 0.95,
        ..Default::default()
    };
    ground(&mut scene, 60.0, 6.0, dirt);
    let rock = scene.add_mesh(shapes::sphere(32, 24));
    for (i, (x, z, r)) in [
        (-3.0, -7.0, 0.8),
        (2.5, -10.0, 1.2),
        (-1.0, -15.0, 0.6),
        (4.5, -5.0, 0.5),
    ]
    .into_iter()
    .enumerate()
    {
        put(
            &mut scene,
            rock,
            StandardMaterial {
                color: Vec3::splat(0.12 + 0.03 * i as f32),
                roughness: 0.85,
                ..Default::default()
            },
            Mat4::from_scale_rotation_translation(
                Vec3::new(r * 1.3, r * 0.7, r),
                Quat::from_rotation_y(i as f32),
                Vec3::new(x, r * 0.25, z),
            ),
        );
    }
    scene.surface_weather = Some(SurfaceWeather {
        wetness: 0.45,
        puddles: 0.6,
        rain: 0.0,
        puddle_size: 3.0,
        ..SurfaceWeather::default()
    });
    // The still puddles mirror the rocks and the sky.
    scene.look.post.ssr = Some(ScreenSpaceReflections::default());
    scene.camera = Camera {
        position: Vec3::new(0.0, 1.7, 3.0),
        target: Vec3::new(0.5, 0.0, -12.0),
        exposure: Exposure::Manual { ev: 0.5 },
        ..Default::default()
    };
    scene
}

/// A yard after a night's snowfall under a low winter sun: snow lies
/// thick on the ground, on crates, on a wall's top and its stepped
/// ledges and on rounded rocks, thinner where faces steepen and absent
/// on the walls themselves and under a lean-to's roof.
fn snow_buildup() -> RenderScene {
    let mut scene = RenderScene::new();
    physical_light(&mut scene, 10.0, PhysicalAtmosphere::EARTH);
    let stone = StandardMaterial {
        color: Vec3::new(0.4, 0.38, 0.35),
        map: scene.add_texture(mottled([0.9, 0.9, 0.9], 31)).slot(),
        aoMap: scene.add_texture(relief(37)).slot(),
        aoMapIntensity: 0.4,
        roughness: 0.9,
        ..Default::default()
    };
    ground(&mut scene, 60.0, 6.0, stone);
    let wood = StandardMaterial {
        color: Vec3::new(0.4, 0.27, 0.16),
        roughness: 0.8,
        ..Default::default()
    };
    // A wall with three stepped ledges along its face.
    bx(
        &mut scene,
        Vec3::new(10.0, 3.0, 0.6),
        Vec3::new(-1.0, 1.5, -9.0),
        stone,
    );
    for k in 0..3 {
        bx(
            &mut scene,
            Vec3::new(10.0, 0.12, 0.35),
            Vec3::new(-1.0, 0.7 + 0.8 * k as f32, -8.55),
            stone,
        );
    }
    // Crates stacked and alone.
    bx(
        &mut scene,
        Vec3::splat(1.0),
        Vec3::new(2.6, 0.5, -5.0),
        wood,
    );
    bx(
        &mut scene,
        Vec3::splat(0.8),
        Vec3::new(2.5, 1.4, -5.1),
        wood,
    );
    bx(
        &mut scene,
        Vec3::splat(0.9),
        Vec3::new(-3.5, 0.45, -4.5),
        wood,
    );
    // A lean-to: a roof on posts, dry ground under it.
    bx(
        &mut scene,
        Vec3::new(3.0, 0.12, 2.0),
        Vec3::new(-3.5, 2.2, -6.5),
        wood,
    );
    for x in [-4.9f32, -2.1] {
        bx(
            &mut scene,
            Vec3::new(0.12, 2.2, 0.12),
            Vec3::new(x, 1.1, -5.6),
            wood,
        );
    }
    let rock = scene.add_mesh(shapes::sphere(40, 30));
    for (x, z, r) in [
        (0.4f32, -4.0f32, 0.7f32),
        (-0.8, -6.2, 0.45),
        (4.2, -7.5, 0.9),
    ] {
        put(
            &mut scene,
            rock,
            StandardMaterial {
                color: Vec3::splat(0.3),
                roughness: 0.75,
                ..Default::default()
            },
            Mat4::from_scale_rotation_translation(
                Vec3::new(r, r * 0.8, r),
                Quat::IDENTITY,
                Vec3::new(x, r * 0.3, z),
            ),
        );
    }
    scene.surface_weather = Some(SurfaceWeather {
        snow: 0.75,
        snow_depth: 0.06,
        ..SurfaceWeather::default()
    });
    scene.camera = Camera {
        position: Vec3::new(2.0, 4.3, 3.0),
        target: Vec3::new(-0.5, 0.4, -6.5),
        exposure: Exposure::Manual { ev: -0.3 },
        ..Default::default()
    };
    scene
}

/// A desert yard in the late afternoon after a dust storm: sand lies on
/// every block's top and drifts into the ground's low parts, dulls a
/// once-polished steel drum and a glazed tile, and the blocks' sides stay
/// their own colour.
fn desert() -> RenderScene {
    let mut scene = RenderScene::new();
    physical_light(&mut scene, 16.8, PhysicalAtmosphere::EARTH);
    let packed = StandardMaterial {
        color: Vec3::splat(0.9),
        map: scene.add_texture(mottled([0.55, 0.42, 0.3], 41)).slot(),
        aoMap: scene.add_texture(relief(43)).slot(),
        aoMapIntensity: 0.5,
        roughness: 0.9,
        ..Default::default()
    };
    ground(&mut scene, 80.0, 8.0, packed);
    let sandstone = StandardMaterial {
        color: Vec3::new(0.5, 0.3, 0.2),
        map: scene.add_texture(mottled([0.95, 0.9, 0.85], 47)).slot(),
        roughness: 0.85,
        ..Default::default()
    };
    for (size, at) in [
        (Vec3::new(2.0, 1.2, 2.0), Vec3::new(-2.5, 0.6, -6.0)),
        (Vec3::new(1.2, 2.4, 1.2), Vec3::new(-2.0, 1.2, -9.5)),
        (Vec3::new(4.0, 0.5, 1.5), Vec3::new(2.0, 0.25, -8.0)),
        (Vec3::new(1.0, 0.8, 1.0), Vec3::new(3.3, 0.9, -8.0)),
    ] {
        bx(&mut scene, size, at, sandstone);
    }
    // A polished steel ball resting on the sand (the sphere's radius is
    // 0.5), a glazed tile slab.
    let drum = scene.add_mesh(shapes::sphere(48, 32));
    put(
        &mut scene,
        drum,
        StandardMaterial {
            color: Vec3::new(0.8, 0.8, 0.82),
            metalness: 1.0,
            roughness: 0.2,
            ..Default::default()
        },
        Mat4::from_scale_rotation_translation(
            Vec3::splat(1.1),
            Quat::IDENTITY,
            Vec3::new(-1.6, 0.55, -2.6),
        ),
    );
    bx(
        &mut scene,
        Vec3::new(1.4, 0.1, 1.4),
        Vec3::new(1.8, 0.05, -3.8),
        StandardMaterial {
            color: Vec3::new(0.05, 0.25, 0.45),
            roughness: 0.1,
            ..Default::default()
        },
    );
    scene.surface_weather = Some(SurfaceWeather {
        dust: 0.8,
        dust_color: Vec3::new(0.86, 0.74, 0.56),
        ..SurfaceWeather::default()
    });
    scene.camera = Camera {
        position: Vec3::new(1.5, 4.0, 2.5),
        target: Vec3::new(-0.3, 0.3, -7.0),
        exposure: Exposure::Manual { ev: -0.5 },
        ..Default::default()
    };
    scene
}

/// A sheet `w` by `h` metres hung from its top edge in vertical folds that
/// deepen towards the hem, as a curtain or a cape (normals from the
/// surface, uvs over the sheet).
fn drape(w: f32, h: f32) -> MeshData {
    let (nx, ny) = (96u32, 64u32);
    let at = |u: f32, v: f32| {
        // Broad, rounded folds a hand and a half apart, deepening towards
        // the hem, with a slower swell across them (a hung curtain).
        let depth = 0.05 + 0.06 * v;
        let x = (u - 0.5) * w;
        let z = depth * (u * 5.0 * std::f32::consts::TAU).sin()
            + 0.3 * depth * (u * 9.0 * std::f32::consts::TAU + v * 2.0).sin()
            + 0.06 * (u * 1.5 * std::f32::consts::TAU + 0.8).sin();
        Vec3::new(x * (1.0 - 0.06 * v), h * (1.0 - v), z)
    };
    let mut vertices = Vec::new();
    for j in 0..=ny {
        for i in 0..=nx {
            let (u, v) = (i as f32 / nx as f32, j as f32 / ny as f32);
            let e = 1e-3;
            let du = at(u + e, v) - at(u - e, v);
            let dv = at(u, v + e) - at(u, v - e);
            let n = dv.cross(du).normalize();
            vertices.push(Vertex {
                position: at(u, v).to_array(),
                normal: n.to_array(),
                uv: [u, v],
                color: [1.0; 4],
                tangent: [0.0; 4],
            });
        }
    }
    let mut indices = Vec::new();
    for j in 0..ny {
        for i in 0..nx {
            let a = j * (nx + 1) + i;
            let b = a + nx + 1;
            indices.extend([a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    MeshData { vertices, indices }
}

/// Crimson velvet: a dark, saturated base under a bright, soft sheen that
/// glows where the folds turn away from the eye.
pub fn velvet() -> StandardMaterial {
    StandardMaterial {
        color: Vec3::new(0.07, 0.004, 0.012),
        roughness: 1.0,
        sheen: 1.0,
        sheenColor: Vec3::new(0.9, 0.5, 0.55),
        sheenRoughness: 0.5,
        side: 2,
        ..Default::default()
    }
}

/// Blue cotton: matte, a faint pale fuzz at its silhouette.
pub fn cotton() -> StandardMaterial {
    StandardMaterial {
        color: Vec3::new(0.09, 0.15, 0.34),
        roughness: 1.0,
        sheen: 1.0,
        sheenColor: Vec3::new(0.08, 0.1, 0.14),
        sheenRoughness: 0.9,
        side: 2,
        ..Default::default()
    }
}

/// Velvet and cotton under the afternoon sky: a crimson velvet drape and a
/// blue cotton one hung side by side, a ball of each on the ground in
/// front, the set turned so the sun comes from behind the camera and to
/// its side. Velvet's tight sheen lights the folds' turned-away faces in a
/// bright rim over a dark face; cotton's rough sheen is a faint fuzz at
/// its silhouette over a matte cloth, never a highlight.
fn cloth() -> RenderScene {
    cloth_with(velvet(), cotton())
}

/// The cloth scene with its two materials (velvet's on the left).
pub fn cloth_with(left: StandardMaterial, right: StandardMaterial) -> RenderScene {
    let mut scene = RenderScene::new();
    physical_light(&mut scene, 15.5, PhysicalAtmosphere::EARTH);
    scene.fog = None;
    ground(
        &mut scene,
        40.0,
        1.0,
        StandardMaterial {
            color: Vec3::splat(0.35),
            roughness: 0.8,
            ..Default::default()
        },
    );
    // The set's +z (towards the camera) turned 40 degrees off the sun's
    // bearing: lit from over the viewer's shoulder.
    let sun = scene.light.direction_to_light;
    let turn = Quat::from_rotation_y(sun.x.atan2(sun.z) + 0.7);
    let sheet = scene.add_mesh(drape(2.2, 2.6));
    let ball = scene.add_mesh(shapes::sphere(64, 48));
    for (x, m) in [(-1.3f32, left), (1.3, right)] {
        put(
            &mut scene,
            sheet,
            m,
            Mat4::from_rotation_translation(turn, turn * Vec3::new(x, 0.02, -1.5)),
        );
        // (The sphere's radius is 0.5: scaled to 0.4, resting on the ground.)
        put(
            &mut scene,
            ball,
            m,
            Mat4::from_scale_rotation_translation(
                Vec3::splat(0.8),
                turn,
                turn * Vec3::new(x, 0.4, 0.2),
            ),
        );
    }
    scene.camera = Camera {
        position: turn * Vec3::new(0.0, 1.4, 4.2),
        target: turn * Vec3::new(0.0, 1.0, -1.0),
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    };
    scene
}

/// T4's displaced cobbles over their crest in the rain (a terrain): the
/// mortar joints, the layer's low relief, fill with water first and
/// mirror the sky between the stones, whose tops stay dark and wet under
/// the running film, the drops splashing on them.
fn cobbles_wet() -> RenderScene {
    let mut scene = crate::terrain_scenes::displaced_scene(crate::terrain_scenes::DISPLACED_RELIEF);
    scene.surface_weather = Some(SurfaceWeather {
        wetness: 1.0,
        puddles: 0.55,
        rain: 1.0,
        puddle_size: 1.5,
        shelter: 0.0,
        ..SurfaceWeather::default()
    });
    scene
}
