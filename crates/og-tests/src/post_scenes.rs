//! The post and antialiasing scenes as shots of a game: the old town's
//! park railings with a boulder rolling down the lane, its high street and
//! market square at noon, dusk and night. Where a test measures a scene it
//! keeps its own plain fixture (`scenes::thin_geometry` for the TAA, TAAU,
//! FSR and SMAA T2x tests, `fx::dof_scene` for the depth of field's, and
//! `scenes::spike_gltf_scene` for `gltf-reference`); the shots here are the
//! registered scenes.

use crate::kit;
use crate::lighting_scenes::{high_street, light_lanterns, market_square, night, wares};
use crate::sky_scenes::physical_light_under;
use glam::{Mat4, Quat, Vec2, Vec3};
use og_render::{
    Camera, DepthOfField, DofQuality, Exposure, LightDesc, LightKind, Material, MotionBlur,
    RenderScene, shapes,
};

/// The AA scenes (`taa`, `fxaa`, `smaa`, `smaa-t2x`, `taau`, their moving
/// and sharpened variants): a lane past a park's wrought-iron railings on a
/// bright noon, the bars 2 cm wide, telegraph wires across the sky and the
/// terrace's eaves behind; a 2 m boulder on the lane, the last instance,
/// which `shadow_scenes::pan_and_roll_sphere` rolls. Framed about x = 0,
/// which `pan_thin_geometry` slides.
pub fn railings_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    let setts = kit::surface_tinted(&mut scene, "cobbles", [0.88, 0.86, 0.82]);
    let lane = scene.add_mesh(kit::slab(
        Vec2::new(-60.0, -0.6),
        Vec2::new(60.0, 40.0),
        0.0,
        1.6,
    ));
    scene.add_instance(lane, setts, Mat4::IDENTITY);
    let grass = kit::surface_tinted(&mut scene, "grass", [0.86, 1.0, 0.72]);
    for (min, max) in [
        (Vec2::new(-260.0, -260.0), Vec2::new(260.0, -0.6)),
        (Vec2::new(-260.0, -0.6), Vec2::new(-60.0, 260.0)),
        (Vec2::new(60.0, -0.6), Vec2::new(260.0, 260.0)),
        (Vec2::new(-60.0, 40.0), Vec2::new(60.0, 260.0)),
    ] {
        let m = scene.add_mesh(kit::slab(min, max, 0.0, 7.0));
        scene.add_instance(m, grass, Mat4::IDENTITY);
    }
    kit::hills(&mut scene, None);
    let iron = scene.add_material(Material {
        base_color: [0.03, 0.03, 0.035, 1.0],
        metallic: 0.6,
        roughness: 0.45,
        ..Default::default()
    });
    // The railings: a low stone plinth, bars every 12 cm with a ball
    // finial each, two rails.
    let stone = kit::surface_tinted(&mut scene, "stone", [0.9, 0.86, 0.8]);
    let plinth = scene.add_mesh(kit::block(
        Vec3::new(-30.0, 0.0, -1.0),
        Vec3::new(30.0, 0.35, -0.6),
        1.2,
    ));
    scene.add_instance(plinth, stone, Mat4::IDENTITY);
    let bar = scene.add_mesh(shapes::cuboid(Vec3::new(0.02, 1.7, 0.02)));
    let knob = scene.add_mesh(shapes::sphere(8, 6));
    for i in 0..500 {
        let x = -30.0 + 0.12 * i as f32;
        scene.add_instance(
            bar,
            iron,
            Mat4::from_translation(Vec3::new(x, 0.35 + 0.85, -0.8)),
        );
        scene.add_instance(
            knob,
            iron,
            Mat4::from_translation(Vec3::new(x, 2.07, -0.8)) * Mat4::from_scale(Vec3::splat(0.05)),
        );
    }
    let rail = scene.add_mesh(shapes::cuboid(Vec3::new(60.0, 0.03, 0.03)));
    for y in [0.55, 1.85] {
        scene.add_instance(rail, iron, Mat4::from_translation(Vec3::new(0.0, y, -0.8)));
    }
    // The park behind them: pines, and the terrace across it.
    let pine = kit::prop(&mut scene, "pine");
    for k in 0..14 {
        let h = |n: i32| crate::fx_scenes::hash2(k, n, 0x7a1);
        pine.place(
            &mut scene,
            Vec3::new(-24.0 + 3.6 * k as f32 + h(0) * 2.0, 0.0, -9.0 - h(1) * 8.0),
            h(2) * 6.0,
            7.0 + h(3) * 5.0,
        );
    }
    let house = kit::prop(&mut scene, "townhouse");
    for k in 0..9 {
        let height = 11.5 + 2.5 * ((k as f32 * 2.3).sin() * 0.5 + 0.5);
        house.place(
            &mut scene,
            Vec3::new(-31.0 + 7.8 * k as f32, 0.0, -30.0),
            0.0,
            height,
        );
    }
    // Telegraph poles along the lane's far side and their four wires,
    // sagging and crossing the sky at a slant.
    let wood = kit::surface_tinted(&mut scene, "planks", [0.55, 0.45, 0.38]);
    let pole = scene.add_mesh(kit::block(
        Vec3::new(-0.09, 0.0, -0.09),
        Vec3::new(0.09, 7.5, 0.09),
        2.0,
    ));
    let arm = scene.add_mesh(shapes::cuboid(Vec3::new(0.08, 0.08, 1.4)));
    let wire = scene.add_mesh(shapes::cuboid(Vec3::new(1.0, 0.012, 0.012)));
    let poles = [-22.0f32, -8.0, 6.0, 20.0];
    for (k, x) in poles.iter().enumerate() {
        let z = -2.0 - 0.8 * k as f32;
        scene.add_instance(pole, wood, Mat4::from_translation(Vec3::new(*x, 0.0, z)));
        scene.add_instance(arm, wood, Mat4::from_translation(Vec3::new(*x, 7.1, z)));
    }
    for k in 0..poles.len() - 1 {
        for w in 0..4 {
            let off = -0.6 + 0.4 * w as f32;
            let a = Vec3::new(poles[k], 7.15, -2.0 - 0.8 * k as f32 + off);
            let b = Vec3::new(poles[k + 1], 7.15, -2.0 - 0.8 * (k + 1) as f32 + off);
            // Each span as eight straight pieces along a sag of 0.5 m.
            for s in 0..8 {
                let at = |t: f32| a.lerp(b, t) - Vec3::Y * (0.5 * 4.0 * t * (1.0 - t));
                let (p, q) = (at(s as f32 / 8.0), at((s + 1) as f32 / 8.0));
                let d = q - p;
                let turn = Quat::from_rotation_arc(Vec3::X, d.normalize());
                scene.add_instance(
                    wire,
                    iron,
                    Mat4::from_scale_rotation_translation(
                        Vec3::new(d.length(), 1.0, 1.0),
                        turn,
                        (p + q) * 0.5,
                    ),
                );
            }
        }
    }
    physical_light_under(&mut scene, Vec3::new(0.45, 0.8, 0.35));
    // The boulder, last.
    // The kit's rock is a terrain layer: its colour and normal, rough.
    let (rock_color, rock_normal) = kit::layer_maps(&mut scene, "rock");
    let rock = scene.add_material(Material {
        base_color_texture: Some(rock_color),
        normal_texture: Some(rock_normal),
        roughness: 0.9,
        ..Default::default()
    });
    let ball = scene.add_mesh(shapes::sphere(48, 32));
    scene.add_instance(
        ball,
        rock,
        Mat4::from_translation(Vec3::new(2.0, 1.0, 1.0)) * Mat4::from_scale(Vec3::splat(2.0)),
    );
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 1.6, 6.0),
        target: Vec3::new(0.0, 3.0, -4.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    };
    scene
}

/// The lanterns' colours (linear, emissive): orange, ice blue, green,
/// white and magenta, as bright as the bloom scenes need.
const LANTERNS: [[f32; 3]; 5] = [
    [8.0, 2.0, 0.5],
    [0.5, 3.0, 10.0],
    [2.0, 9.0, 3.0],
    [12.0, 12.0, 12.0],
    [10.0, 0.5, 6.0],
];

/// The bloom and tone mapping scenes (`bloom`, `bloom-unreal`,
/// `post-agx`, `post-neutral`, `post-blur`): the high street on a
/// festival night, five paper lanterns in saturated colours hung across
/// it on a line, each lighting the setts under it in its colour, the
/// street lamps lit warm down to the watchtower.
pub fn lantern_night_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    let lamps = high_street(&mut scene, 90.0);
    wares(&mut scene, Vec3::new(-2.4, 0.0, -1.0), 1.4);
    wares(&mut scene, Vec3::new(2.7, 0.0, -9.0), -1.7);
    night(&mut scene);
    light_lanterns(&mut scene, &lamps[2..], 6.0, 0, 3.0);
    let globe = scene.add_mesh(shapes::sphere(32, 24));
    let cord = scene.add_mesh(shapes::cuboid(Vec3::new(1.0, 0.015, 0.015)));
    let dark = scene.add_material(Material {
        base_color: [0.05, 0.04, 0.035, 1.0],
        roughness: 0.8,
        ..Default::default()
    });
    for (i, c) in LANTERNS.iter().enumerate() {
        let x = -3.0 + 1.5 * i as f32;
        let p = Vec3::new(x, 3.2 + 0.45 * (i % 2) as f32, -1.5 - 1.1 * i as f32);
        let m = scene.add_material(Material {
            base_color: [0.0, 0.0, 0.0, 1.0],
            emissive: *c,
            ..Default::default()
        });
        scene.add_instance(
            globe,
            m,
            Mat4::from_scale_rotation_translation(Vec3::new(0.55, 0.65, 0.55), Quat::IDENTITY, p),
        );
        // Its cord up to the line across the street.
        scene.add_instance(
            cord,
            dark,
            Mat4::from_translation(p + Vec3::Y * 0.6)
                * Mat4::from_rotation_z(std::f32::consts::FRAC_PI_2)
                * Mat4::from_scale(Vec3::new(0.6, 1.0, 1.0)),
        );
        let c = Vec3::from(*c);
        scene.lighting.lights.push(LightDesc {
            kind: LightKind::Point,
            position: p - Vec3::Y * 0.5,
            color: c / c.max_element(),
            intensity: 12.0,
            range: 10.0,
            radius: 0.0,
            cast_shadows: false,
            mobility: og_render::LightMobility::Movable,
        });
    }
    let line = scene.add_mesh(kit::block(
        Vec3::new(-4.4, 4.45, -0.02),
        Vec3::new(4.4, 4.47, 0.02),
        1.0,
    ));
    scene.add_instance(
        line,
        dark,
        Mat4::from_translation(Vec3::new(0.0, 0.0, -3.7)) * Mat4::from_rotation_y(-0.6),
    );
    scene.frame.camera = Camera {
        position: Vec3::new(0.4, 1.7, 6.5),
        target: Vec3::new(0.0, 2.6, -8.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    };
    scene
}

/// `dof-low`, `dof-high`: a fruit seller's counter on the high street at
/// dusk seen from beside a barrel, the counter in focus 5 m off, the
/// barrel at the camera's shoulder and the lamps down the street out of
/// focus, their glass spread into bokeh.
pub fn dof_market_scene(quality: DofQuality) -> RenderScene {
    let mut scene = RenderScene::new();
    let lamps = high_street(&mut scene, 120.0);
    let counter = Vec3::new(-0.6, 0.0, 1.0);
    wares(&mut scene, counter, 0.35);
    wares(&mut scene, Vec3::new(2.6, 0.0, -12.0), -1.7);
    kit::prop(&mut scene, "winebarrel").place(&mut scene, Vec3::new(2.0, 0.0, 5.2), 0.6, 1.0);
    physical_light_under(&mut scene, Vec3::new(-0.7, 0.04, -0.71));
    light_lanterns(&mut scene, &lamps, 8.0, 2, 4.0);
    let eye = Vec3::new(1.4, 1.35, 6.0);
    let subject = counter + Vec3::new(0.0, 1.0, 0.0);
    scene.look.post.bloom = Some(og_render::Bloom::default());
    scene.look.post.depth_of_field = Some(DepthOfField {
        focus_distance: eye.distance(subject),
        focus_range: 1.2,
        max_blur: 0.014,
        falloff: 3.0,
        tilt_shift: 0.0,
        quality,
    });
    scene.frame.camera = Camera {
        position: eye,
        target: subject + Vec3::new(0.0, 0.2, -1.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: -0.6 },
        ..Default::default()
    };
    scene
}

/// The car's length and the barrel's radius (at [`street_blur_scene`]'s
/// heights).
const CAR_HEIGHT: f32 = 1.45;
const BARREL_HEIGHT: f32 = 1.0;
const BARREL_RADIUS: f32 = BARREL_HEIGHT * 0.58 / 1.9;

/// The car driving across the square at `frame`: 0.7 m a frame along +x.
fn car_at(frame: u32) -> Mat4 {
    Mat4::from_translation(Vec3::new(-9.0 + 0.7 * frame as f32, 0.0, -5.0))
}

/// A barrel rolling across the setts towards the camera's right at
/// `frame`, turning by the distance it rolls.
fn barrel_at(frame: u32) -> Mat4 {
    let x = -2.6 + 0.35 * frame as f32;
    Mat4::from_translation(Vec3::new(x, BARREL_RADIUS, 1.6))
        * Mat4::from_rotation_z(-(x / BARREL_RADIUS))
        * Mat4::from_rotation_x(std::f32::consts::FRAC_PI_2)
        * Mat4::from_translation(Vec3::new(0.0, -0.5 * BARREL_HEIGHT, 0.0))
}

fn street_blur_camera(frame: u32) -> Camera {
    Camera {
        position: Vec3::new(frame as f32 * 0.25, 1.8, 7.0),
        target: Vec3::new(frame as f32 * 0.25, 1.6, -4.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    }
}

/// `motion-blur`: the market square at noon, a car driving across it and
/// a barrel come loose rolling over the setts, the camera panning with
/// them ([`move_street_blur`]): per-object and camera motion blur. The car
/// is instance 0 and the barrel instance 1.
pub fn street_blur_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    let (car, paint) = kit::prop(&mut scene, "car").rooted(&mut scene, CAR_HEIGHT);
    scene.add_instance(car, paint, car_at(0));
    let (barrel, oak) = kit::prop(&mut scene, "winebarrel").rooted(&mut scene, BARREL_HEIGHT);
    scene.add_instance(barrel, oak, barrel_at(0));
    market_square(&mut scene);
    wares(&mut scene, Vec3::new(-6.0, 0.0, -10.0), 0.3);
    wares(&mut scene, Vec3::new(5.0, 0.0, -12.0), -0.4);
    physical_light_under(&mut scene, Vec3::new(0.35, 0.8, 0.45));
    scene.look.post.motion_blur = Some(MotionBlur::default());
    scene.frame.camera = street_blur_camera(0);
    scene
}

/// Moves [`street_blur_scene`]'s car, barrel and camera to `frame`, with
/// their previous transforms for the per-object motion vectors.
pub fn move_street_blur(scene: &mut RenderScene, frame: u32) {
    let prev = frame.saturating_sub(1);
    scene.objects.previous_transforms = scene
        .objects
        .instances
        .iter()
        .map(|i| i.transform)
        .collect();
    scene.objects.previous_transforms[0] = car_at(prev);
    scene.objects.previous_transforms[1] = barrel_at(prev);
    scene.objects.instances[0].transform = car_at(frame);
    scene.objects.instances[1].transform = barrel_at(frame);
    scene.frame.camera = street_blur_camera(frame);
    crate::fx_scenes::tick_all(scene, frame as u64);
}

/// `post-sharpen`: the high street at noon, 200 m of setts down to the
/// watchtower, their far courses softened by the texture's mips and
/// sharpened back (CAS at strength 1).
pub fn sharpen_street_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    high_street(&mut scene, 200.0);
    wares(&mut scene, Vec3::new(-2.4, 0.0, -3.0), 1.4);
    physical_light_under(&mut scene, Vec3::new(-0.3, 0.85, 0.4));
    scene.look.post.sharpen = Some(1.0);
    scene.frame.camera = Camera {
        position: Vec3::new(0.6, 1.5, 7.0),
        target: Vec3::new(0.0, 1.0, -60.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.2 },
        ..Default::default()
    };
    scene
}

/// The market square with a stall's wares by the camera, lit by a sun
/// towards `sun` (night below the horizon, its lamps lit), seen from
/// `camera`: `post-grade` and `post-adjust` at golden hour, `post-lens`
/// at noon through a wide lens, `post-crt` at dusk.
pub fn square_scene(sun: Vec3, camera: Camera) -> RenderScene {
    let mut scene = RenderScene::new();
    market_square(&mut scene);
    wares(&mut scene, Vec3::new(-3.0, 0.0, 6.0), 0.5);
    wares(&mut scene, Vec3::new(6.5, 0.0, -2.0), -0.9);
    wares(&mut scene, Vec3::new(-9.0, 0.0, -14.0), 0.2);
    let lamp = kit::prop(&mut scene, "streetlamp");
    let mut glass = Vec::new();
    for (x, z) in [(-12.0, 4.0), (2.0, -6.0), (14.0, 3.0), (-4.0, -20.0)] {
        let foot = Vec3::new(x, 0.0, z);
        lamp.place(&mut scene, foot, 0.0, 4.2);
        glass.push(crate::lighting_scenes::Lantern {
            glass: foot + Vec3::Y * 3.35,
            side: 1.0,
        });
    }
    physical_light_under(&mut scene, sun);
    if sun.y < 0.15 {
        light_lanterns(&mut scene, &glass, 10.0, 1, 3.0);
    }
    scene.frame.camera = Camera {
        far: 2500.0,
        ..camera
    };
    scene
}

/// [`square_scene`]'s shots: golden hour from the stall (`post-grade`,
/// `post-adjust`), noon through a 100° lens (`post-lens`), dusk (`post-crt`).
pub fn square_golden() -> RenderScene {
    square_scene(
        Vec3::new(0.1, 0.42, 0.9),
        Camera {
            position: Vec3::new(-6.5, 1.7, 10.0),
            target: Vec3::new(4.0, 3.0, -20.0),
            exposure: Exposure::Manual { ev: -0.2 },
            ..Default::default()
        },
    )
}

pub fn square_wide() -> RenderScene {
    square_scene(
        Vec3::new(0.3, 0.85, 0.45),
        Camera {
            position: Vec3::new(0.0, 1.6, 12.0),
            target: Vec3::new(0.0, 4.0, -20.0),
            projection: og_render::Projection::Perspective {
                fov_y_degrees: 80.0,
            },
            exposure: Exposure::Manual { ev: 0.3 },
            ..Default::default()
        },
    )
}

pub fn square_dusk() -> RenderScene {
    square_scene(
        Vec3::new(-0.7, 0.02, -0.71),
        Camera {
            position: Vec3::new(4.0, 2.0, 14.0),
            target: Vec3::new(-2.0, 3.0, -20.0),
            exposure: Exposure::Manual { ev: 1.2 },
            ..Default::default()
        },
    )
}
