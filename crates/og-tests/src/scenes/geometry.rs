//! Ground, lighting, exposure and antialiasing scenes and shared geometry textures.

use super::*;

/// Rows top to bottom of an RGBA8 image made by `f(x, y) -> [r, g, b, a]`.
pub(crate) fn texture(
    size: u32,
    color_space: ColorSpace,
    f: impl Fn(f32, f32) -> [u8; 4],
) -> TextureData {
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let u = (x as f32 + 0.5) / size as f32;
            let v = (y as f32 + 0.5) / size as f32;
            rgba.extend_from_slice(&f(u, v));
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

pub(crate) fn ground(scene: &mut RenderScene, size: f32, cell_m: f32, roughness: f32) {
    let checker = scene.add_texture(shapes::checker(512, 16));
    let material = scene.add_material(Material {
        base_color_texture: Some(checker),
        roughness,
        ..Default::default()
    });
    let mesh = scene.add_mesh(shapes::plane(size, size / (16.0 * cell_m)));
    scene.add_instance(mesh, material, Mat4::IDENTITY);
}

/// One tile of a ground grid: a dark slate fill with a pale line along its
/// edges (two texels each side, so tiled copies meet in a four-texel line).
pub fn grid_tile(size: u32) -> TextureData {
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let edge = |c: u32| c < 2 || c >= size - 2;
            let px = if edge(x) || edge(y) {
                [150, 200, 210, 255]
            } else {
                [38, 44, 52, 255]
            };
            rgba.extend_from_slice(&px);
        }
    }
    TextureData {
        width: size,
        height: size,
        rgba,
        color_space: ColorSpace::Srgb,
        mips: Mips::Generate,
    }
}

/// A 1,600 m ground built as og.render.planeGeometry builds it (the xy
/// plane turned -π/2 about x) with its UVs repeated `repeat` times
/// (`uvRepeat`): at 160 the grid tile lies every 10 m. Red posts stand on
/// the grid lines every 10 m down the middle, so each cell's length reads
/// against them.
pub fn ground_grid(repeat: f32) -> RenderScene {
    let mut scene = RenderScene::new();
    let tile = scene.add_texture(grid_tile(128));
    let ground = scene.add_material(Material {
        base_color_texture: Some(tile),
        roughness: 0.9,
        ..Default::default()
    });
    let mut plane = shapes::plane_xy(1600.0, 1600.0);
    shapes::repeat_uv(&mut plane, Vec2::splat(repeat));
    let plane = scene.add_mesh(plane);
    scene.add_instance(
        plane,
        ground,
        Mat4::from_rotation_x(-std::f32::consts::FRAC_PI_2),
    );
    let post = scene.add_mesh(shapes::cuboid(Vec3::new(0.4, 2.0, 0.4)));
    let red = scene.add_material(Material {
        base_color: [0.8, 0.15, 0.1, 1.0],
        roughness: 0.6,
        ..Default::default()
    });
    for k in 1..=8 {
        scene.add_instance(
            post,
            red,
            Mat4::from_translation(Vec3::new(0.0, 1.0, -10.0 * k as f32)),
        );
    }
    scene.camera = Camera {
        position: Vec3::new(4.0, 6.0, 4.0),
        target: Vec3::new(0.0, 0.0, -35.0),
        far: 2000.0,
        exposure: Exposure::linear(0.8),
        ..Default::default()
    };
    let sun = Vec3::new(0.5, 0.7, 0.3);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 4.0,
        ..Default::default()
    };
    scene.environment.sky.sun_direction = sun;
    scene
}

/// The 400 m field. `tall` adds the 8 m pillars and the 20 m wall; without
/// them only short casters remain (0.5 m blocks, 0.6 m kerbs), whose shadows
/// a bias too large for their size would lose.
pub fn cascades_200m(tall: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    ground(&mut scene, 420.0, 1.0, 0.9);
    let pillar = scene.add_mesh(shapes::cuboid(Vec3::new(1.0, 8.0, 1.0)));
    let block = scene.add_mesh(shapes::cuboid(Vec3::splat(0.5)));
    let wall = scene.add_mesh(shapes::cuboid(Vec3::new(30.0, 20.0, 2.0)));
    let stone = scene.add_material(Material {
        base_color: [0.75, 0.72, 0.68, 1.0],
        roughness: 0.8,
        ..Default::default()
    });
    let red = scene.add_material(Material {
        base_color: [0.8, 0.15, 0.1, 1.0],
        roughness: 0.6,
        ..Default::default()
    });
    // Kerbs running away from the camera: the sun throws their shadows
    // sideways onto the ground in view.
    let kerb = scene.add_mesh(shapes::cuboid(Vec3::new(1.0, 0.6, 30.0)));
    for d in [12.0f32, 32.0, 80.0, 150.0] {
        scene.add_instance(
            kerb,
            red,
            Mat4::from_translation(Vec3::new(1.5 + d * 0.05, 0.3, -d)),
        );
    }
    let pillars: &[f32] = if tall {
        &[
            3.0, 6.0, 10.0, 16.0, 24.0, 35.0, 50.0, 70.0, 95.0, 125.0, 160.0, 190.0,
        ]
    } else {
        &[]
    };
    for (i, d) in pillars.iter().enumerate() {
        let x = if i % 2 == 0 { -5.0 } else { 5.0 } * (1.0 + d / 60.0);
        scene.add_instance(pillar, stone, Mat4::from_translation(Vec3::new(x, 4.0, -d)));
    }
    // Small blocks near the camera: shadows of small objects in cascade 0.
    for i in 0..6 {
        let z = -3.0 - i as f32 * 0.9;
        scene.add_instance(
            block,
            red,
            Mat4::from_translation(Vec3::new(-1.2, 0.25, z)) * Mat4::from_rotation_y(i as f32),
        );
    }
    if tall {
        scene.add_instance(
            wall,
            stone,
            Mat4::from_translation(Vec3::new(-20.0, 10.0, -200.0)),
        );
    }
    scene.camera = Camera {
        position: Vec3::new(0.0, 5.0, 6.0),
        target: Vec3::new(0.0, 0.0, -45.0),
        far: 1000.0,
        exposure: Exposure::linear(0.8),
        ..Default::default()
    };
    let sun = Vec3::new(0.8, 0.45, 0.25);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 4.0,
        ..Default::default()
    };
    scene.environment.sky.sun_direction = sun;
    scene
}

pub fn point_spot_lights() -> RenderScene {
    let mut scene = RenderScene::new();
    ground(&mut scene, 30.0, 0.5, 0.7);
    let pillar = scene.add_mesh(shapes::cuboid(Vec3::new(0.6, 3.0, 0.6)));
    let sphere = scene.add_mesh(shapes::sphere(32, 24));
    let white = scene.add_material(Material {
        base_color: [0.85, 0.85, 0.85, 1.0],
        roughness: 0.5,
        ..Default::default()
    });
    let metal = scene.add_material(Material {
        base_color: [0.9, 0.8, 0.6, 1.0],
        metallic: 1.0,
        roughness: 0.3,
        ..Default::default()
    });
    for (x, z) in [
        (-3.0, -2.0),
        (2.5, -3.0),
        (0.0, 2.5),
        (-4.0, 3.0),
        (4.5, 1.5),
    ] {
        scene.add_instance(pillar, white, Mat4::from_translation(Vec3::new(x, 1.5, z)));
    }
    for (x, z) in [(-1.2, -0.5), (1.5, 0.3), (0.2, -2.0)] {
        scene.add_instance(sphere, metal, Mat4::from_translation(Vec3::new(x, 0.5, z)));
    }
    scene.lights = vec![
        // Shadowed: a spot from above-left, and a point light among the pillars.
        Light {
            kind: LightKind::Spot {
                direction: Vec3::new(0.5, -1.0, 0.2),
                angle_degrees: 35.0,
                penumbra: 0.3,
            },
            position: Vec3::new(-4.0, 6.0, -1.0),
            color: Vec3::new(1.0, 0.95, 0.85),
            intensity: 90.0,
            range: 20.0,
            cast_shadows: true,
        },
        Light {
            position: Vec3::new(0.3, 1.2, 0.0),
            color: Vec3::new(1.0, 0.7, 0.4),
            intensity: 12.0,
            range: 12.0,
            cast_shadows: true,
            ..Default::default()
        },
        Light {
            position: Vec3::new(-5.0, 0.8, -5.0),
            color: Vec3::new(1.0, 0.1, 0.1),
            intensity: 8.0,
            range: 7.0,
            ..Default::default()
        },
        Light {
            position: Vec3::new(5.0, 0.8, -5.0),
            color: Vec3::new(0.1, 1.0, 0.2),
            intensity: 8.0,
            range: 7.0,
            ..Default::default()
        },
        Light {
            position: Vec3::new(5.0, 0.8, 5.0),
            color: Vec3::new(0.2, 0.3, 1.0),
            intensity: 8.0,
            range: 7.0,
            ..Default::default()
        },
        Light {
            position: Vec3::new(-6.0, 0.6, 5.0),
            color: Vec3::new(1.0, 0.5, 1.0),
            intensity: 6.0,
            range: 6.0,
            ..Default::default()
        },
        Light {
            kind: LightKind::Spot {
                direction: Vec3::new(-1.0, -0.6, -0.2),
                angle_degrees: 20.0,
                penumbra: 0.6,
            },
            position: Vec3::new(8.0, 3.0, 2.0),
            color: Vec3::new(0.3, 1.0, 1.0),
            intensity: 60.0,
            range: 18.0,
            cast_shadows: false,
        },
    ];
    scene.light.intensity = 0.0;
    scene.light.cast_shadows = false;
    scene.environment = Environment {
        intensity: 0.04,
        background: Background::Color(Vec3::new(0.004, 0.006, 0.015)),
        ..Default::default()
    };
    scene.camera = Camera {
        position: Vec3::new(0.0, 7.0, 11.0),
        target: Vec3::new(0.0, 0.5, 0.0),
        exposure: Exposure::linear(1.2),
        ..Default::default()
    };
    scene
}

/// Thin poles, a sphere and a fine checker at a grazing angle: aliasing.
pub fn thin_geometry() -> RenderScene {
    let mut scene = RenderScene::new();
    ground(&mut scene, 60.0, 0.25, 0.8);
    let pole = scene.add_mesh(shapes::cuboid(Vec3::new(0.04, 4.0, 0.04)));
    let bar = scene.add_mesh(shapes::cuboid(Vec3::new(6.0, 0.05, 0.05)));
    let sphere = scene.add_mesh(shapes::sphere(32, 24));
    let dark = scene.add_material(Material {
        base_color: [0.05, 0.05, 0.06, 1.0],
        roughness: 0.4,
        ..Default::default()
    });
    for i in 0..12 {
        let x = -6.0 + i as f32 * 1.1;
        let z = -2.0 - i as f32 * 0.6;
        scene.add_instance(
            pole,
            dark,
            Mat4::from_translation(Vec3::new(x, 2.0, z)) * Mat4::from_rotation_z(0.15),
        );
    }
    for i in 0..5 {
        scene.add_instance(
            bar,
            dark,
            Mat4::from_translation(Vec3::new(0.0, 0.6 + i as f32 * 0.5, -1.0))
                * Mat4::from_rotation_z(0.08 * i as f32),
        );
    }
    let shiny = scene.add_material(Material {
        base_color: [0.9, 0.9, 0.95, 1.0],
        metallic: 1.0,
        roughness: 0.15,
        ..Default::default()
    });
    scene.add_instance(
        sphere,
        shiny,
        Mat4::from_translation(Vec3::new(2.0, 1.0, 1.0)) * Mat4::from_scale(Vec3::splat(2.0)),
    );
    scene.camera = Camera {
        position: Vec3::new(0.0, 1.6, 6.0),
        target: Vec3::new(0.0, 1.2, -4.0),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    scene
}

pub(super) fn emissive_spheres() -> RenderScene {
    let mut scene = RenderScene::new();
    ground(&mut scene, 30.0, 0.5, 0.6);
    let sphere = scene.add_mesh(shapes::sphere(32, 24));
    for (i, c) in [
        [8.0, 2.0, 0.5],
        [0.5, 3.0, 10.0],
        [2.0, 9.0, 3.0],
        [12.0, 12.0, 12.0],
        [10.0, 0.5, 6.0],
    ]
    .iter()
    .enumerate()
    {
        let m = scene.add_material(Material {
            base_color: [0.0, 0.0, 0.0, 1.0],
            emissive: *c,
            ..Default::default()
        });
        let x = -4.0 + i as f32 * 2.0;
        scene.add_instance(
            sphere,
            m,
            Mat4::from_translation(Vec3::new(x, 0.6 + (i % 2) as f32 * 0.8, -(i as f32) * 0.7))
                * Mat4::from_scale(Vec3::splat(0.8 + 0.2 * (i % 3) as f32)),
        );
    }
    scene.light.intensity = 0.3;
    scene.environment = Environment {
        intensity: 0.08,
        background: Background::Color(Vec3::new(0.01, 0.012, 0.02)),
        ..Default::default()
    };
    scene.camera = Camera {
        position: Vec3::new(0.0, 3.0, 8.0),
        target: Vec3::new(0.0, 0.8, -1.0),
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    scene
}

/// A dusk scene lit `brightness` times: metered exposure brings it to middle grey.
pub fn dim_scene(brightness: f32) -> RenderScene {
    let mut scene = RenderScene::new();
    ground(&mut scene, 40.0, 1.0, 0.8);
    let sphere = scene.add_mesh(shapes::sphere(32, 24));
    let block = scene.add_mesh(shapes::cuboid(Vec3::new(1.5, 2.5, 1.5)));
    let warm = scene.add_material(Material {
        base_color: [0.9, 0.5, 0.25, 1.0],
        roughness: 0.4,
        ..Default::default()
    });
    let cool = scene.add_material(Material {
        base_color: [0.3, 0.5, 0.8, 1.0],
        roughness: 0.7,
        ..Default::default()
    });
    for i in 0..5 {
        let x = -6.0 + i as f32 * 3.0;
        scene.add_instance(sphere, warm, Mat4::from_translation(Vec3::new(x, 0.5, 0.0)));
        scene.add_instance(
            block,
            cool,
            Mat4::from_translation(Vec3::new(x + 1.5, 1.25, -4.0)),
        );
    }
    let sun = Vec3::new(-0.6, 0.25, 0.4);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 0.15 * brightness,
        ..Default::default()
    };
    scene.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            sun_color: Vec3::new(1.0, 0.7, 0.45),
            look: None,
        },
        intensity: 0.03 * brightness,
        background: Background::Color(Vec3::new(0.01, 0.012, 0.02) * brightness),
        ..Environment::default()
    };
    scene.camera = Camera {
        position: Vec3::new(0.0, 3.0, 10.0),
        target: Vec3::new(0.0, 1.0, -1.0),
        exposure: Exposure::auto(),
        ..Default::default()
    };
    scene
}
