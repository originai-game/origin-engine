//! Shadow and antialiasing scenes of the shadows-aa round
//! (`crates/og-render/README.md` §Shadows and antialiasing by tier): the
//! sun's soft shadows with cascade blending, eight shadowed local lights,
//! and the thin geometry under TAA with motion, Catmull-Rom history and
//! sharpening.

use crate::scenes::{ground, load};
use glam::{Mat4, Quat, Vec3};
use og_render::{
    Background, Camera, DirectionalLight, Environment, Exposure, Light, LightKind, Material,
    RenderScene, shapes,
};

/// The sun's PCSS softness and cascade blend of `shadows-soft`.
pub const SOFTNESS: f32 = 0.03;
pub const CASCADE_BLEND: f32 = 0.2;

/// A low sun over a long field: a cube on the ground (contact shadows stay
/// sharp), a plank on two posts 2.5 m up and a ball floating at 3 m (their
/// penumbrae widen with the height), and a row of thin poles running away
/// from the camera across the cascade splits. `soft` turns PCSS and the
/// cascade blend on.
pub fn soft_shadows_scene(soft: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    ground(&mut scene, 160.0, 1.0, 0.9);
    let stone = scene.add_material(Material {
        base_color: [0.78, 0.74, 0.68, 1.0],
        roughness: 0.8,
        ..Default::default()
    });
    let red = scene.add_material(Material {
        base_color: [0.75, 0.16, 0.1, 1.0],
        roughness: 0.5,
        ..Default::default()
    });
    let cube = scene.add_mesh(shapes::cuboid(Vec3::splat(1.2)));
    let post = scene.add_mesh(shapes::cuboid(Vec3::new(0.25, 2.5, 0.25)));
    let plank = scene.add_mesh(shapes::cuboid(Vec3::new(4.0, 0.15, 1.2)));
    let ball = scene.add_mesh(shapes::sphere(32, 24));
    let pole = scene.add_mesh(shapes::cuboid(Vec3::new(0.15, 5.0, 0.15)));
    scene.add_instance(cube, red, Mat4::from_translation(Vec3::new(-3.5, 0.6, 1.0)));
    for x in [-1.6f32, 1.6] {
        scene.add_instance(
            post,
            stone,
            Mat4::from_translation(Vec3::new(1.5 + x, 1.25, -1.0)),
        );
    }
    scene.add_instance(
        plank,
        stone,
        Mat4::from_translation(Vec3::new(1.5, 2.575, -1.0)),
    );
    scene.add_instance(
        ball,
        red,
        Mat4::from_scale_rotation_translation(
            Vec3::splat(1.4),
            Quat::IDENTITY,
            Vec3::new(-1.0, 3.0, -5.0),
        ),
    );
    for i in 0..10 {
        let z = -9.0 - i as f32 * 6.0;
        let x = if i % 2 == 0 { -3.0 } else { 3.0 };
        scene.add_instance(pole, stone, Mat4::from_translation(Vec3::new(x, 2.5, z)));
    }
    let sun = Vec3::new(0.75, 0.42, 0.3);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 4.0,
        max_shadow_distance: 90.0,
        shadow_softness: if soft { SOFTNESS } else { 0.0 },
        cascade_blend: if soft { CASCADE_BLEND } else { 0.0 },
        ..Default::default()
    };
    scene.environment.sky.sun_direction = sun;
    scene.environment.intensity = 0.45;
    scene.camera = Camera {
        position: Vec3::new(0.0, 5.5, 9.0),
        target: Vec3::new(0.0, 0.0, -14.0),
        exposure: Exposure::linear(0.85),
        ..Default::default()
    };
    scene
}

/// Night over a plaza: eight shadowed lights (six point, two spot), each
/// beside a pillar, the brightest nearest the camera, plus two more
/// shadowed point lights far behind (dim at the camera: past the tier's
/// eight, their shadows are the ones left out). `extra` adds those two.
pub fn many_shadowed_lights(extra: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    ground(&mut scene, 60.0, 0.5, 0.7);
    let pillar = scene.add_mesh(shapes::cuboid(Vec3::new(0.5, 2.4, 0.5)));
    let white = scene.add_material(Material {
        base_color: [0.85, 0.85, 0.85, 1.0],
        roughness: 0.6,
        ..Default::default()
    });
    let colors = [
        Vec3::new(1.0, 0.55, 0.25),
        Vec3::new(0.3, 0.6, 1.0),
        Vec3::new(0.35, 1.0, 0.4),
        Vec3::new(1.0, 0.3, 0.7),
        Vec3::new(1.0, 0.9, 0.4),
        Vec3::new(0.6, 0.4, 1.0),
        Vec3::new(1.0, 1.0, 1.0),
        Vec3::new(0.3, 1.0, 1.0),
    ];
    let mut lights = Vec::new();
    for (i, color) in colors.iter().enumerate() {
        let col = (i % 4) as f32;
        let row = (i / 4) as f32;
        let at = Vec3::new(-6.0 + col * 4.0, 0.0, -1.0 - row * 5.0);
        scene.add_instance(
            pillar,
            white,
            Mat4::from_translation(at + Vec3::new(0.0, 1.2, 0.0)),
        );
        let kind = if i == 3 || i == 6 {
            LightKind::Spot {
                direction: Vec3::new(0.35, -1.0, -0.3),
                angle_degrees: 38.0,
                penumbra: 0.3,
            }
        } else {
            LightKind::Point
        };
        let position = if matches!(kind, LightKind::Spot { .. }) {
            at + Vec3::new(-1.0, 3.6, 0.8)
        } else {
            at + Vec3::new(-0.9, 1.0, 0.9)
        };
        lights.push(Light {
            kind,
            position,
            color: *color,
            intensity: if matches!(kind, LightKind::Spot { .. }) {
                40.0
            } else {
                9.0
            },
            range: 7.0,
            cast_shadows: true,
        });
    }
    if extra {
        for x in [-8.0f32, 8.0] {
            let at = Vec3::new(x, 0.0, -24.0);
            scene.add_instance(
                pillar,
                white,
                Mat4::from_translation(at + Vec3::new(0.0, 1.2, 0.0)),
            );
            lights.push(Light {
                position: at + Vec3::new(-0.9, 1.0, 0.9),
                color: Vec3::new(1.0, 0.8, 0.6),
                intensity: 6.0,
                range: 7.0,
                cast_shadows: true,
                ..Default::default()
            });
        }
    }
    scene.lights = lights;
    scene.light.intensity = 0.0;
    scene.light.cast_shadows = false;
    scene.environment = Environment {
        intensity: 0.03,
        background: Background::Color(Vec3::new(0.004, 0.006, 0.015)),
        ..Default::default()
    };
    scene.camera = Camera {
        position: Vec3::new(0.0, 8.5, 9.0),
        target: Vec3::new(0.0, 0.0, -5.0),
        exposure: Exposure::linear(1.1),
        ..Default::default()
    };
    scene
}

/// The thin geometry's camera sliding sideways, 2 cm a frame: TAA's
/// history is resampled every frame, where Catmull-Rom keeps it sharper
/// than a bilinear tap.
pub fn pan_thin_geometry(scene: &mut RenderScene, frame: u32) {
    let x = frame as f32 * 0.02;
    scene.camera.position.x = x;
    scene.camera.target.x = x;
}

/// A roofed stone room (a crypt) on a lit field: four walls with a door in
/// the east one, a roof slab, an altar and a pillar inside, a low sun
/// outside that reaches the floor only through the door, and a shadowed point
/// light hanging inside in a glowing glass box that casts no shadow
/// (`Instance::cast_shadows` false: The Kestrel Stair's crypt lamp, web
/// findings #4), so the light reaches the altar and the pillar and throws a
/// post's shadow beside them. The camera stands inside by the door,
/// looking west at the altar.
pub fn crypt_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    ground(&mut scene, 60.0, 1.0, 0.9);
    let stone = scene.add_material(Material {
        base_color: [0.72, 0.66, 0.58, 1.0],
        roughness: 0.85,
        ..Default::default()
    });
    let red = scene.add_material(Material {
        base_color: [0.7, 0.14, 0.1, 1.0],
        roughness: 0.6,
        ..Default::default()
    });
    // The room: 8 m square inside, walls 0.6 m thick and 4 m high, a roof
    // 0.4 m thick on them; the door is 2.4 m wide and 3 m high.
    let (half, t, h) = (4.0f32, 0.6f32, 4.0f32);
    let slab = |scene: &mut RenderScene, size: Vec3, at: Vec3| {
        let mesh = scene.add_mesh(shapes::cuboid(size));
        scene.add_instance(mesh, stone, Mat4::from_translation(at));
    };
    let outer = 2.0 * (half + t);
    slab(
        &mut scene,
        Vec3::new(outer, h, t),
        Vec3::new(0.0, h / 2.0, -half - t / 2.0),
    );
    slab(
        &mut scene,
        Vec3::new(outer, h, t),
        Vec3::new(0.0, h / 2.0, half + t / 2.0),
    );
    slab(
        &mut scene,
        Vec3::new(t, h, 2.0 * half),
        Vec3::new(-half - t / 2.0, h / 2.0, 0.0),
    );
    let (door, door_h) = (1.2f32, 3.0f32);
    let side = half - door;
    for z in [-(door + side / 2.0), door + side / 2.0] {
        slab(
            &mut scene,
            Vec3::new(t, h, side),
            Vec3::new(half + t / 2.0, h / 2.0, z),
        );
    }
    slab(
        &mut scene,
        Vec3::new(t, h - door_h, 2.0 * door),
        Vec3::new(half + t / 2.0, (h + door_h) / 2.0, 0.0),
    );
    slab(
        &mut scene,
        Vec3::new(outer, 0.4, outer),
        Vec3::new(0.0, h + 0.2, 0.0),
    );
    // Inside: an altar against the west wall, a pillar, and a post between
    // the light and the pillar.
    slab(
        &mut scene,
        Vec3::new(1.4, 1.1, 2.2),
        Vec3::new(-3.0, 0.55, 0.0),
    );
    slab(
        &mut scene,
        Vec3::new(0.8, h, 0.8),
        Vec3::new(-1.6, h / 2.0, -2.2),
    );
    let post = scene.add_mesh(shapes::cuboid(Vec3::new(0.3, 1.6, 0.3)));
    scene.add_instance(
        post,
        red,
        Mat4::from_translation(Vec3::new(-0.2, 0.8, -1.4)),
    );
    // A low sun from the east: outside it lights the field, inside only
    // the patch the door lets in; the roof and the walls shadow the rest.
    let sun = Vec3::new(0.75, 0.42, 0.3).normalize();
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 4.0,
        max_shadow_distance: 70.0,
        ..Default::default()
    };
    scene.environment.sky.sun_direction = sun;
    scene.environment.intensity = 0.12;
    let lamp = Vec3::new(0.4, 2.2, 0.6);
    let glass = scene.add_material(Material {
        base_color: [1.0, 0.8, 0.4, 1.0],
        emissive: [4.0, 2.4, 1.0],
        ..Default::default()
    });
    let lantern = scene.add_mesh(shapes::cuboid(Vec3::new(0.22, 0.3, 0.22)));
    scene.instances.push(og_render::Instance {
        mesh: lantern,
        material: glass,
        transform: Mat4::from_translation(lamp),
        material_layer: None,
        fx: og_render::InstanceFx::NONE,
        cast_shadows: false,
        receive_shadows: true,
    });
    scene.lights = vec![Light {
        position: lamp,
        color: Vec3::new(1.0, 0.62, 0.3),
        intensity: 7.0,
        range: 9.0,
        cast_shadows: true,
        ..Default::default()
    }];
    scene.camera = Camera {
        position: Vec3::new(3.4, 2.4, 2.6),
        target: Vec3::new(-3.0, 0.9, -0.6),
        exposure: Exposure::linear(1.6),
        ..Default::default()
    };
    scene
}

/// Where `shadow-flags` puts its objects, for the tests to toggle them:
/// indices into the scene's instances (a model's are a range).
pub struct FlagsScene {
    pub scene: RenderScene,
    /// The front trickster added as a model is (casting).
    pub model_as_is: std::ops::Range<usize>,
    /// The front trickster with `cast_shadows` off.
    pub model_no_cast: std::ops::Range<usize>,
    /// The wall.
    pub wall: usize,
    /// Behind the wall, in its shadow: a box and a trickster that receive,
    /// and a box and a trickster with `receive_shadows` off.
    pub box_receives: usize,
    pub model_receives: std::ops::Range<usize>,
    pub box_no_receive: usize,
    pub model_no_receive: std::ops::Range<usize>,
}

/// Shadow flags per instance (`Instance::cast_shadows` and
/// `receive_shadows`; og.Mesh's and og.Model's castShadow and
/// receiveShadow): a low sun from the east over a field. In front, on open
/// ground, two tricksters: the left one added as a model is (it casts) and
/// the right one with `cast_shadows` off (no shadow beside it). Behind
/// them a 5 m wall on the east side throws its shadow over a row of four:
/// a red box and a trickster that receive (dark in the wall's shadow),
/// then a red box and a trickster with `receive_shadows` off, lit as if
/// the wall were not there while still casting on the ground.
pub fn shadow_flags_scene() -> Result<FlagsScene, String> {
    let mut scene = RenderScene::new();
    ground(&mut scene, 60.0, 1.0, 0.9);
    let stone = scene.add_material(Material {
        base_color: [0.72, 0.68, 0.62, 1.0],
        roughness: 0.85,
        ..Default::default()
    });
    let red = scene.add_material(Material {
        base_color: [0.75, 0.16, 0.1, 1.0],
        roughness: 0.5,
        ..Default::default()
    });
    let model = load("trickster.glb")?;
    let parts = model.add_to(&mut scene);
    let scale = 1.8 / (model.max - model.min).max_element().max(1e-3);
    let trickster = |scene: &mut RenderScene, x: f32, z: f32, yaw: f32| {
        let first = scene.instances.len();
        parts.instance(
            scene,
            Mat4::from_translation(Vec3::new(x, -model.min.y * scale, z))
                * Mat4::from_rotation_y(yaw)
                * Mat4::from_scale(Vec3::splat(scale)),
        );
        first..scene.instances.len()
    };
    let model_as_is = trickster(&mut scene, -1.6, 2.2, 0.4);
    let model_no_cast = trickster(&mut scene, 2.4, 2.2, -0.4);
    let wall = scene.instances.len();
    let slab = scene.add_mesh(shapes::cuboid(Vec3::new(0.5, 5.0, 6.0)));
    scene.add_instance(
        slab,
        stone,
        Mat4::from_translation(Vec3::new(3.6, 2.5, -3.2)),
    );
    let cube = scene.add_mesh(shapes::cuboid(Vec3::splat(1.0)));
    let block = |scene: &mut RenderScene, x: f32| {
        let i = scene.instances.len();
        scene.add_instance(
            cube,
            red,
            Mat4::from_translation(Vec3::new(x, 0.5, -3.0)) * Mat4::from_rotation_y(0.35),
        );
        i
    };
    let box_receives = block(&mut scene, 2.3);
    let model_receives = trickster(&mut scene, 1.0, -3.0, 0.2);
    let box_no_receive = block(&mut scene, -0.4);
    let model_no_receive = trickster(&mut scene, -1.8, -3.0, 0.2);
    for i in model_no_cast.clone() {
        scene.instances[i].cast_shadows = false;
    }
    for i in model_no_receive.clone().chain([box_no_receive]) {
        scene.instances[i].receive_shadows = false;
    }
    let sun = Vec3::new(0.85, 0.48, 0.12).normalize();
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 4.0,
        max_shadow_distance: 60.0,
        ..Default::default()
    };
    scene.environment.sky.sun_direction = sun;
    scene.environment.intensity = 0.3;
    scene.camera = Camera {
        position: Vec3::new(0.2, 3.6, 8.6),
        target: Vec3::new(0.4, 0.7, -1.2),
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    Ok(FlagsScene {
        scene,
        model_as_is,
        model_no_cast,
        wall,
        box_receives,
        model_receives,
        box_no_receive,
        model_no_receive,
    })
}
