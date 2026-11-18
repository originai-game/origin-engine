//! Weather on surfaces and cloth (Wave B): a rain-wet street at dusk with
//! its lamps lit, puddles drying after the rain, snow built up on ledges,
//! crates and rocks, a dusty desert yard, and velvet and cotton drapes
//! under the sky. Each frame is drawn from a [`SurfaceWeather`] the way the
//! runtime fills it from og.WeatherState: a scene sets what lies on its
//! surfaces, and where it rains the rain is og.Weather's
//! ([`crate::atmo_scenes::add_weather`]), which brings the drops on the
//! puddles, the closed sky and the dimmed light with it.

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
    scene.atmosphere.clouds = Some(Clouds {
        coverage,
        color: Vec3::splat(0.7),
        base: 900.0,
        layer: 0.0,
        ..Clouds::default()
    });
}

/// A street at dusk in the rain: an old town's setts between kerbed
/// pavements, its ochre townhouses shoulder to shoulder on both sides,
/// iron lamps lit along the kerbs and a little lacquered city car parked
/// by one. The setts are soaked and dark, their hollows hold rippling
/// puddles that mirror the lamps and the house fronts, the rain runs down
/// the walls and the car's lacquer beads.
fn street() -> RenderScene {
    use crate::kit;
    let mut scene = RenderScene::new();
    overcast(&mut scene, 18.5, 0.8);
    let setts = kit::surface_tinted(&mut scene, "cobbles", [0.8, 0.78, 0.76]);
    let road = scene.add_mesh(kit::slab(
        glam::Vec2::new(-4.0, -120.0),
        glam::Vec2::new(4.0, 12.0),
        0.0,
        2.0,
    ));
    scene.add_instance(road, setts, Mat4::IDENTITY);
    let concrete = StandardMaterial {
        color: Vec3::new(0.42, 0.41, 0.39),
        map: scene.add_texture(mottled([0.9, 0.9, 0.88], 7)).slot(),
        aoMap: scene.add_texture(relief(13)).slot(),
        aoMapIntensity: 0.4,
        roughness: 0.92,
        ..Default::default()
    };
    let house = kit::prop(&mut scene, "townhouse");
    let lamp = kit::prop(&mut scene, "streetlamp");
    let bulb = scene.add_mesh(shapes::sphere(12, 8));
    let glow = scene.add_standard_material(StandardMaterial {
        color: Vec3::ZERO,
        emissive: Vec3::new(1.0, 0.72, 0.45),
        emissiveIntensity: 12.0,
        weathering: 0.0,
        ..Default::default()
    });
    for side in [-1.0f32, 1.0] {
        // The pavement, its kerb 15 cm up.
        bx(
            &mut scene,
            Vec3::new(2.4, 0.15, 132.0),
            Vec3::new(side * 5.2, 0.075, -54.0),
            concrete,
        );
        // The houses' fronts on the pavement's back edge, facing the
        // street, each a little taller or lower than its neighbour.
        for k in 0..16 {
            let z = 6.0 - 7.8 * k as f32 + if side > 0.0 { 3.4 } else { 0.0 };
            let height = 11.5 + 2.5 * ((k as f32 * 2.3 + side).sin() * 0.5 + 0.5);
            house.place(
                &mut scene,
                Vec3::new(side * 8.75, 0.15, z),
                -side * std::f32::consts::FRAC_PI_2,
                height,
            );
        }
        for k in 0..5 {
            // An iron lamp on the kerb, its lantern lit.
            let z = -2.0 - 14.0 * k as f32 + if side > 0.0 { 7.0 } else { 0.0 };
            let foot = Vec3::new(side * 4.4, 0.15, z);
            lamp.place(&mut scene, foot, 0.0, 4.2);
            let at = foot + Vec3::Y * 3.35;
            scene.add_instance(
                bulb,
                glow,
                Mat4::from_scale_rotation_translation(Vec3::splat(0.16), Quat::IDENTITY, at),
            );
            scene.lighting.lights.push(Light {
                kind: LightKind::Point,
                position: at - Vec3::new(side * 0.3, 0.2, 0.0),
                color: Vec3::new(1.0, 0.72, 0.45),
                intensity: 40.0,
                range: 18.0,
                radius: 0.0,
                cast_shadows: false,
            });
        }
    }
    // A lacquered city car parked by the kerb: its clearcoat sealed, no
    // pores to fill (the model's own KHR_materials_clearcoat).
    kit::prop(&mut scene, "car").place(&mut scene, Vec3::new(2.6, 0.0, -11.0), 1.62, 1.45);
    scene.atmosphere.surface_weather = Some(SurfaceWeather {
        wetness: 1.0,
        puddles: 0.75,
        puddle_size: 2.5,
        ..SurfaceWeather::default()
    });
    // The puddles and the lacquer mirror the lamps and the house fronts.
    scene.look.post.ssr = Some(ScreenSpaceReflections::default());
    scene.frame.camera = Camera {
        position: Vec3::new(-1.4, 1.6, 6.0),
        target: Vec3::new(0.8, 2.4, -30.0),
        exposure: Exposure::Manual { ev: 0.5 },
        ..Default::default()
    };
    crate::atmo_scenes::add_weather(&mut scene, og_render::WEATHER_RAIN, 1.0, Vec3::ZERO, 18.0);
    scene
}

/// A farmyard half an hour after the rain under a clearing sky: the
/// puddles on its packed earth stand still and mirror the sky and the
/// cloud, and the yard dries in patches from its high spots, the hollows
/// and the shores still dark. Weathered boulders, crates and barrels by a
/// dry-stone wall, pines and a watchtower past it, the hills beyond.
fn puddles() -> RenderScene {
    use crate::kit;
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
    ground(&mut scene, 400.0, 40.0, dirt);
    let stone = kit::surface_tinted(&mut scene, "cobbles", [1.0, 0.93, 0.8]);
    for (min, max) in [
        (Vec3::new(-22.0, -0.2, -19.0), Vec3::new(-1.5, 1.1, -18.3)),
        (Vec3::new(2.5, -0.2, -19.0), Vec3::new(22.0, 1.1, -18.3)),
    ] {
        let m = scene.add_mesh(kit::block(min, max, 1.6));
        scene.add_instance(m, stone, Mat4::IDENTITY);
    }
    let boulder = kit::prop(&mut scene, "boulder");
    for (x, z, yaw, h) in [
        (-3.2f32, -7.0f32, 0.3f32, 0.9f32),
        (2.8, -10.5, 1.7, 1.3),
        (-1.2, -15.0, 2.9, 0.7),
        (4.6, -5.2, 4.1, 0.55),
    ] {
        boulder.place(&mut scene, Vec3::new(x, -0.1, z), yaw, h);
    }
    let crate_ = kit::prop(&mut scene, "crate");
    let barrel = kit::prop(&mut scene, "barrel");
    crate_.place(&mut scene, Vec3::new(-6.5, 0.0, -16.6), 0.1, 0.62);
    crate_.place(&mut scene, Vec3::new(-6.4, 0.62, -16.6), 0.25, 0.62);
    crate_.place(&mut scene, Vec3::new(-5.0, 0.0, -16.9), -0.2, 0.62);
    barrel.place(&mut scene, Vec3::new(-8.0, 0.0, -17.0), 0.0, 1.0);
    barrel.place(&mut scene, Vec3::new(-8.9, 0.0, -16.4), 1.0, 1.0);
    let pine = kit::prop(&mut scene, "pine");
    for k in 0..40u32 {
        let h = |i: u32| {
            let mut v = k.wrapping_mul(0x8da6_b343) ^ i.wrapping_mul(0xd816_3841);
            v = (v ^ (v >> 13)).wrapping_mul(0x85eb_ca6b);
            ((v ^ (v >> 16)) & 0xffff) as f32 / 65536.0
        };
        let (x, z) = (-70.0 + 140.0 * h(1), -26.0 - 60.0 * h(2));
        pine.place(
            &mut scene,
            Vec3::new(x, -0.2, z),
            6.3 * h(3),
            9.0 + 7.0 * h(4),
        );
    }
    kit::prop(&mut scene, "watchtower").place(&mut scene, Vec3::new(14.0, -0.3, -34.0), 0.8, 13.0);
    kit::hills(&mut scene, None);
    scene.atmosphere.surface_weather = Some(SurfaceWeather {
        wetness: 0.45,
        puddles: 0.6,
        rain: 0.0,
        puddle_size: 3.0,
        ..SurfaceWeather::default()
    });
    // The still puddles mirror the boulders and the sky.
    scene.look.post.ssr = Some(ScreenSpaceReflections::default());
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 1.7, 3.0),
        target: Vec3::new(0.5, 0.6, -14.0),
        exposure: Exposure::Manual { ev: 0.5 },
        far: 3000.0,
        ..Default::default()
    };
    scene
}

/// A farmyard after a night's snowfall under a low winter sun: snow lies
/// thick on the yard, on its crates and barrels, on a dry-stone wall's
/// top and its stepped ledges and on the boulders, thinner where faces
/// steepen, absent on the wall's face and under a lean-to's roof. Pines
/// and the hills stand white beyond.
fn snow_buildup() -> RenderScene {
    use crate::kit;
    let mut scene = RenderScene::new();
    physical_light(&mut scene, 10.0, PhysicalAtmosphere::EARTH);
    let yard = StandardMaterial {
        color: Vec3::new(0.4, 0.38, 0.35),
        map: scene.add_texture(mottled([0.9, 0.9, 0.9], 31)).slot(),
        aoMap: scene.add_texture(relief(37)).slot(),
        aoMapIntensity: 0.4,
        roughness: 0.9,
        ..Default::default()
    };
    ground(&mut scene, 400.0, 40.0, yard);
    let stone = kit::surface_tinted(&mut scene, "cobbles", [1.0, 0.93, 0.8]);
    let block = |scene: &mut RenderScene, min: Vec3, max: Vec3| {
        let m = scene.add_mesh(kit::block(min, max, 1.6));
        scene.add_instance(m, stone, Mat4::IDENTITY);
    };
    // A wall with three stepped ledges along its face.
    block(
        &mut scene,
        Vec3::new(-6.0, 0.0, -9.3),
        Vec3::new(4.0, 3.0, -8.7),
    );
    for k in 0..3 {
        let y = 0.64 + 0.8 * k as f32;
        block(
            &mut scene,
            Vec3::new(-6.0, y, -8.7),
            Vec3::new(4.0, y + 0.12, -8.35),
        );
    }
    let wood = StandardMaterial {
        color: Vec3::new(0.4, 0.27, 0.16),
        roughness: 0.8,
        ..Default::default()
    };
    // A lean-to: a plank roof on posts, dry ground under it.
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
    // Crates stacked and alone, barrels by the wall, one under the roof.
    let crate_ = kit::prop(&mut scene, "crate");
    let barrel = kit::prop(&mut scene, "barrel");
    crate_.place(&mut scene, Vec3::new(2.6, 0.0, -5.0), 0.2, 0.9);
    crate_.place(&mut scene, Vec3::new(2.5, 0.9, -5.1), 0.6, 0.75);
    crate_.place(&mut scene, Vec3::new(1.2, 0.0, -6.8), -0.3, 0.8);
    barrel.place(&mut scene, Vec3::new(-3.6, 0.0, -6.6), 0.0, 1.0);
    barrel.place(&mut scene, Vec3::new(3.3, 0.0, -7.9), 0.8, 1.0);
    barrel.place(&mut scene, Vec3::new(-0.6, 0.0, -8.0), 2.1, 1.0);
    let boulder = kit::prop(&mut scene, "boulder");
    for (x, z, yaw, h) in [
        (0.4f32, -3.6f32, 0.4f32, 0.7f32),
        (-1.2, -5.6, 2.0, 0.5),
        (5.4, -6.5, 3.4, 1.1),
    ] {
        boulder.place(&mut scene, Vec3::new(x, -0.1, z), yaw, h);
    }
    let pine = kit::prop(&mut scene, "pine");
    for k in 0..30u32 {
        let h = |i: u32| {
            let mut v = k.wrapping_mul(0x27d4_eb2d) ^ i.wrapping_mul(0x1656_67b1);
            v = (v ^ (v >> 15)).wrapping_mul(0x85eb_ca6b);
            ((v ^ (v >> 16)) & 0xffff) as f32 / 65536.0
        };
        let (x, z) = (-50.0 + 100.0 * h(1), -14.0 - 50.0 * h(2));
        pine.place(
            &mut scene,
            Vec3::new(x, -0.2, z),
            6.3 * h(3),
            8.0 + 7.0 * h(4),
        );
    }
    kit::hills(&mut scene, None);
    scene.atmosphere.surface_weather = Some(SurfaceWeather {
        snow: 0.75,
        snow_depth: 0.06,
        ..SurfaceWeather::default()
    });
    scene.frame.camera = Camera {
        position: Vec3::new(2.0, 4.3, 3.0),
        target: Vec3::new(-0.5, 1.2, -9.0),
        exposure: Exposure::Manual { ev: -0.3 },
        far: 3000.0,
        ..Default::default()
    };
    scene
}

/// An oasis hamlet's yard in the late afternoon after a dust storm: sand
/// lies on the adobe houses' flat roofs, the crates' and barrels' lids and
/// the low sandstone walls' tops, drifts into the yard's low parts, dulls
/// a once-polished steel drum and a glazed tile, while every wall's face
/// keeps its own colour. Palms lean over the houses.
fn desert() -> RenderScene {
    use crate::kit;
    let mut scene = RenderScene::new();
    physical_light(&mut scene, 16.8, PhysicalAtmosphere::EARTH);
    let packed = StandardMaterial {
        color: Vec3::new(1.15, 0.95, 0.72),
        map: scene.add_texture(mottled([0.55, 0.42, 0.3], 41)).slot(),
        aoMap: scene.add_texture(relief(43)).slot(),
        aoMapIntensity: 0.5,
        roughness: 0.9,
        ..Default::default()
    };
    ground(&mut scene, 400.0, 40.0, packed);
    let sandstone = StandardMaterial {
        color: Vec3::new(0.5, 0.3, 0.2),
        map: scene.add_texture(mottled([0.95, 0.9, 0.85], 47)).slot(),
        roughness: 0.85,
        ..Default::default()
    };
    // The yard's low walls and a well kerb.
    for (size, at) in [
        (Vec3::new(7.0, 0.9, 0.5), Vec3::new(-4.5, 0.45, -3.5)),
        (Vec3::new(0.5, 0.9, 5.0), Vec3::new(4.8, 0.45, -6.0)),
        (Vec3::new(1.6, 0.6, 1.6), Vec3::new(0.6, 0.3, -9.0)),
    ] {
        bx(&mut scene, size, at, sandstone);
    }
    let adobe = kit::prop(&mut scene, "adobe");
    // The skirt the model stands on sinks into the yard.
    for (x, z, yaw, h) in [
        (-4.6f32, -11.5f32, 0.25f32, 4.2f32),
        (3.6, -14.5, -0.35, 4.6),
        (-12.0, -18.0, 0.6, 4.0),
        (11.0, -20.0, -0.7, 3.8),
    ] {
        adobe.place(&mut scene, Vec3::new(x, -0.15, z), yaw, h);
    }
    let palm = kit::prop(&mut scene, "palm");
    for (x, z, yaw, h) in [
        (-0.8f32, -15.5f32, 0.4f32, 7.5f32),
        (7.5, -11.0, 2.2, 6.4),
        (-8.5, -9.0, 4.0, 7.0),
        (-16.0, -24.0, 1.1, 8.0),
        (15.0, -26.0, 5.0, 7.2),
    ] {
        palm.place(&mut scene, Vec3::new(x, 0.0, z), yaw, h);
    }
    let crate_ = kit::prop(&mut scene, "crate");
    let barrel = kit::prop(&mut scene, "barrel");
    crate_.place(&mut scene, Vec3::new(2.4, 0.0, -7.0), 0.3, 0.6);
    crate_.place(&mut scene, Vec3::new(2.5, 0.6, -7.05), 0.1, 0.6);
    crate_.place(&mut scene, Vec3::new(3.6, 0.0, -6.4), -0.4, 0.6);
    barrel.place(&mut scene, Vec3::new(-2.8, 0.0, -8.4), 0.0, 1.0);
    barrel.place(&mut scene, Vec3::new(-3.6, 0.0, -7.6), 1.4, 1.0);
    // A polished steel drum standing in the yard, a glazed tile slab.
    let drum = scene.add_mesh(og_render::geometry::primitives::cylinder(
        &og_render::geometry::primitives::Cylinder {
            radius_top: 0.35,
            radius_bottom: 0.35,
            height: 0.95,
            radial_segments: 48,
            ..Default::default()
        },
    ));
    put(
        &mut scene,
        drum,
        StandardMaterial {
            color: Vec3::new(0.8, 0.8, 0.82),
            metalness: 1.0,
            roughness: 0.2,
            ..Default::default()
        },
        Mat4::from_translation(Vec3::new(-1.4, 0.475, -5.2)),
    );
    bx(
        &mut scene,
        Vec3::new(1.4, 0.1, 1.4),
        Vec3::new(1.0, 0.05, -4.6),
        StandardMaterial {
            color: Vec3::new(0.05, 0.25, 0.45),
            roughness: 0.1,
            ..Default::default()
        },
    );
    scene.atmosphere.surface_weather = Some(SurfaceWeather {
        dust: 0.8,
        dust_color: Vec3::new(0.86, 0.74, 0.56),
        ..SurfaceWeather::default()
    });
    scene.frame.camera = Camera {
        position: Vec3::new(1.0, 3.2, 3.0),
        target: Vec3::new(-0.3, 1.6, -12.0),
        exposure: Exposure::Manual { ev: -0.5 },
        far: 3000.0,
        ..Default::default()
    };
    // The yard turned to face away from the low sun in the west, which
    // stands behind the camera's left shoulder: the house fronts and the
    // yard lie in its light, their shadows thrown back away from the view.
    let turn = Mat4::from_rotation_y(-2.29);
    for i in &mut scene.objects.instances {
        i.transform = turn * i.transform;
    }
    let camera = &mut scene.frame.camera;
    camera.position = turn.transform_point3(camera.position);
    camera.target = turn.transform_point3(camera.target);
    scene
}

/// A sheet `w` by `h` metres hung from its top edge in vertical folds that
/// deepen towards the hem, as a curtain or a cape (normals from the
/// surface, uvs over the sheet).
fn drape(w: f32, h: f32) -> MeshData {
    drape_grid(w, h, 96, 64)
}

/// [`drape`] over `nx` by `ny` cells.
fn drape_grid(w: f32, h: f32, nx: u32, ny: u32) -> MeshData {
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

/// A market stall's valance: a strip of red and cream striped canvas
/// `w` metres long, `h` deep, hung in folds from its top edge at its
/// origin (stripes across it, 25 cm each).
pub fn valance(
    scene: &mut RenderScene,
    w: f32,
    h: f32,
) -> (og_render::MeshHandle, og_render::MaterialHandle) {
    let stripes = (w / 0.25).round().max(1.0);
    let canvas = texture(256, ColorSpace::Srgb, |u, v| {
        let weave = 0.92 + 0.08 * fbm(u, v, 32, 53);
        let red = [0.62, 0.1, 0.08];
        let cream = [0.86, 0.8, 0.66];
        let c = if (u * 2.0).fract() < 0.5 { red } else { cream };
        // The hem's band a shade darker.
        let hem = if v > 0.82 { 0.8 } else { 1.0 };
        c.map(|x| x * weave * hem)
    });
    let mut mesh = drape_grid(w, h, (stripes as u32) * 6, 8);
    for v in &mut mesh.vertices {
        v.uv[0] *= stripes / 2.0;
    }
    let mesh = scene.add_mesh(mesh);
    let map = scene.add_texture(canvas).slot();
    let material = scene.add_standard_material(StandardMaterial {
        color: Vec3::ONE,
        map,
        roughness: 0.9,
        sheen: 0.6,
        sheenColor: Vec3::splat(0.12),
        sheenRoughness: 0.8,
        side: 2,
        ..Default::default()
    });
    (mesh, material)
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
    scene.atmosphere.fog = None;
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
    let sun = scene.lighting.light.direction_to_light;
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
    scene.frame.camera = Camera {
        position: turn * Vec3::new(0.0, 1.4, 4.2),
        target: turn * Vec3::new(0.0, 1.0, -1.0),
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    };
    scene
}

/// T4's displaced cobbles over their crest in the rain (a terrain), the
/// old way's ruined colonnade on the crest and pines beyond it: the
/// mortar joints, the layer's low relief, fill with water first and
/// mirror the sky between the stones, whose tops stay dark and wet under
/// the running film, the drops splashing on them.
fn cobbles_wet() -> RenderScene {
    // The old paved way over its crest (`terrain-displaced-way`'s set:
    // the colonnade's stumps, a watchtower and pines past the crest) under
    // a closing rain sky.
    let mut scene = crate::terrain_scenes::displaced_way();
    if let Some(c) = scene.atmosphere.clouds.as_mut() {
        c.coverage = 0.85;
        c.color = Vec3::splat(0.72);
    }
    scene.atmosphere.surface_weather = Some(SurfaceWeather {
        wetness: 1.0,
        puddles: 0.55,
        puddle_size: 1.5,
        shelter: 0.0,
        ..SurfaceWeather::default()
    });
    // The water in the joints mirrors the sky and the colonnade.
    scene.look.post.ssr = Some(ScreenSpaceReflections::default());
    // Standing on the way below the crest, looking down its rise: the
    // joints between the setts read as lines of standing water.
    {
        use crate::terrain_scenes::displaced_height as h;
        let camera = &mut scene.frame.camera;
        camera.position = Vec3::new(0.0, h(-5.0) + 1.9, -5.0);
        camera.target = Vec3::new(0.2, h(-9.5) - 0.1, -9.5);
    }
    crate::atmo_scenes::add_weather(&mut scene, og_render::WEATHER_RAIN, 1.0, Vec3::ZERO, 18.0);
    scene
}
