//! Shadow and antialiasing scenes of the shadows-aa round
//! (`crates/og-render/README.md` §Shadows and antialiasing): the
//! sun's soft shadows with cascade blending, eight shadowed local lights,
//! and the thin geometry under TAA with motion, Catmull-Rom history and
//! sharpening.

use crate::scenes::load;
use glam::{Mat4, Vec3};
use og_render::{
    Background, Camera, DirectionalLight, Environment, Exposure, Light, LightKind, Material,
    RenderScene, shapes,
};

/// The sun's PCSS softness and cascade blend of `shadows-soft`.
pub const SOFTNESS: f32 = 0.03;
pub const CASCADE_BLEND: f32 = 0.2;

/// A market stall in the old town's square on a summer afternoon: crates
/// and barrels on the setts (their contact shadows stay sharp), the
/// stall's plank roof and canvas 2.5 to 3.2 m up on its posts (its shadow's
/// edge softens with the height), and a row of street lamps running away
/// across the square through the cascade splits. `soft` turns PCSS and the cascade
/// blend on.
pub fn soft_shadows_scene(soft: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    crate::lighting_scenes::market_square(&mut scene);
    crate::sky_scenes::canopy(&mut scene, false);
    let lamp = crate::kit::prop(&mut scene, "streetlamp");
    for k in 0..8 {
        lamp.place(
            &mut scene,
            Vec3::new(-7.5, 0.0, 4.0 - 7.0 * k as f32),
            0.0,
            4.2,
        );
    }
    // The sun behind the stall and to its left: the roof's shadow comes
    // towards the camera over the setts, the lamps' shadows run alongside.
    let sun = Vec3::new(-0.5, 0.5, -0.62);
    crate::sky_scenes::physical_light_under(&mut scene, sun);
    let light = &mut scene.lighting.light;
    light.max_shadow_distance = 90.0;
    light.shadow_softness = if soft { SOFTNESS } else { 0.0 };
    light.cascade_blend = if soft { CASCADE_BLEND } else { 0.0 };
    scene.frame.camera = Camera {
        position: Vec3::new(5.5, 1.8, 5.0),
        target: Vec3::new(-0.8, 0.9, -5.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: -0.2 },
        ..Default::default()
    };
    scene
}

/// A night market in the old town's square, lit by eight shadowed lights:
/// six lamps standing round the stall and its wares (point lights in their
/// lanterns) and two spots hung under the stall's roof over its counters;
/// with `extra`, two more lamps at the back row of houses, whose shadows
/// fall past the tier's eight and are left out (they are the farthest and
/// the faintest at the camera). Every crate, barrel, post and lamp throws a
/// shadow from each light near it.
pub fn many_shadowed_lights(extra: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    crate::lighting_scenes::market_square(&mut scene);
    crate::lighting_scenes::night(&mut scene);
    crate::sky_scenes::canopy(&mut scene, false);
    crate::lighting_scenes::wares(&mut scene, Vec3::new(-7.5, 0.0, -1.0), 0.6);
    crate::lighting_scenes::wares(&mut scene, Vec3::new(7.0, 0.0, -2.5), -0.9);
    let lamp = crate::kit::prop(&mut scene, "streetlamp");
    let warm = [
        Vec3::new(1.0, 0.7, 0.42),
        Vec3::new(1.0, 0.62, 0.35),
        Vec3::new(1.0, 0.76, 0.5),
    ];
    let mut lamps = vec![
        Vec3::new(-5.5, 0.0, 1.5),
        Vec3::new(5.5, 0.0, 0.5),
        Vec3::new(-9.5, 0.0, -6.5),
        Vec3::new(9.5, 0.0, -7.5),
        Vec3::new(-3.0, 0.0, -12.5),
        Vec3::new(3.5, 0.0, -13.0),
    ];
    if extra {
        lamps.extend([Vec3::new(-14.0, 0.0, -23.0), Vec3::new(14.0, 0.0, -23.5)]);
    }
    let mut lights = Vec::new();
    for (i, foot) in lamps.iter().enumerate() {
        lamp.place(&mut scene, *foot, 0.0, 4.2);
        lights.push(Light {
            position: *foot + Vec3::new(0.0, 3.1, 0.0),
            color: warm[i % 3],
            intensity: if i < 6 { 70.0 } else { 40.0 },
            range: 13.0,
            cast_shadows: true,
            ..Default::default()
        });
    }
    // The two spots under the roof, over the counters (inserted among the
    // lamps: the nearest and brightest lights keep their shadows).
    for (k, x) in [-1.9f32, 1.9].into_iter().enumerate() {
        lights.insert(
            3 + k,
            Light {
                kind: LightKind::Spot {
                    direction: Vec3::new(0.0, -1.0, -0.35).normalize(),
                    angle_degrees: 50.0,
                    penumbra: 0.4,
                },
                position: Vec3::new(x, 2.85, -6.6),
                color: Vec3::new(1.0, 0.82, 0.6),
                intensity: 120.0,
                range: 8.0,
                cast_shadows: true,
            },
        );
    }
    scene.lighting.lights = lights;
    scene.frame.camera = Camera {
        position: Vec3::new(1.5, 5.2, 11.0),
        target: Vec3::new(0.0, 0.6, -6.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: -1.2 },
        ..Default::default()
    };
    scene
}

/// The thin geometry's camera sliding sideways, 2 cm a frame: TAA's
/// history is resampled every frame, where Catmull-Rom keeps it sharper
/// than a bilinear tap.
pub fn pan_thin_geometry(scene: &mut RenderScene, frame: u32) {
    let x = frame as f32 * 0.02;
    scene.frame.camera.position.x = x;
    scene.frame.camera.target.x = x;
}

/// [`pan_thin_geometry`] with the thin geometry's sphere (its last
/// instance) rolling 8 cm to the right a frame, its previous transform
/// set for the motion vectors.
pub fn pan_and_roll_sphere(scene: &mut RenderScene, frame: u32) {
    pan_thin_geometry(scene, frame);
    let at = |f: u32| {
        Mat4::from_translation(Vec3::new(1.0 + 0.08 * f as f32, 1.0, 1.0))
            * Mat4::from_rotation_z(-0.04 * f as f32)
            * Mat4::from_scale(Vec3::splat(2.0))
    };
    let sphere = scene.objects.instances.len() - 1;
    scene.objects.instances[sphere].transform = at(frame);
    scene.objects.previous_transforms = scene
        .objects
        .instances
        .iter()
        .map(|i| i.transform)
        .collect();
    scene.objects.previous_transforms[sphere] = at(frame.saturating_sub(1));
}

/// A crypt of rubble stone: a vaulted room 8 m square inside, walls 0.6 m
/// thick and 4 m high under a stone roof, a door 2.4 m wide and 3 m high
/// in its east wall; inside, an altar against the west wall, a column on
/// its left, a stone sarcophagus against the north wall, a barrel, and
/// crates by the door.
/// Outside, a meadow under hills. Lit by nothing: the scenes light it.
pub fn crypt_room(scene: &mut RenderScene) {
    use crate::kit;
    let grass = kit::surface_tinted(scene, "grass", [0.86, 1.0, 0.72]);
    let field = scene.add_mesh(kit::slab(
        glam::Vec2::splat(-200.0),
        glam::Vec2::splat(200.0),
        0.0,
        7.0,
    ));
    scene.add_instance(field, grass, Mat4::IDENTITY);
    kit::hills(scene, None);
    let stone = kit::surface_tinted(scene, "stone", [0.92, 0.88, 0.82]);
    let flags = kit::surface_tinted(scene, "cobbles", [0.7, 0.68, 0.64]);
    let (half, t, h) = (4.0f32, 0.6f32, 4.0f32);
    let wall = |scene: &mut RenderScene, min: Vec3, max: Vec3| {
        let mesh = scene.add_mesh(kit::block(min, max, 2.4));
        scene.add_instance(mesh, stone, Mat4::IDENTITY);
    };
    let o = half + t;
    let floor = scene.add_mesh(kit::slab(
        glam::Vec2::splat(-half),
        glam::Vec2::splat(half),
        0.01,
        1.6,
    ));
    scene.add_instance(floor, flags, Mat4::IDENTITY);
    wall(scene, Vec3::new(-o, 0.0, -o), Vec3::new(o, h, -half));
    wall(scene, Vec3::new(-o, 0.0, half), Vec3::new(o, h, o));
    wall(scene, Vec3::new(-o, 0.0, -half), Vec3::new(-half, h, half));
    let (door, door_h) = (1.2f32, 3.0f32);
    wall(scene, Vec3::new(half, 0.0, -half), Vec3::new(o, h, -door));
    wall(scene, Vec3::new(half, 0.0, door), Vec3::new(o, h, half));
    wall(scene, Vec3::new(half, door_h, -door), Vec3::new(o, h, door));
    wall(scene, Vec3::new(-o, h, -o), Vec3::new(o, h + 0.4, o));
    // The altar, its top slab overhanging; the sarcophagus by the north wall.
    wall(scene, Vec3::new(-3.7, 0.0, -1.1), Vec3::new(-2.3, 1.0, 1.1));
    wall(
        scene,
        Vec3::new(-3.8, 1.0, -1.25),
        Vec3::new(-2.15, 1.15, 1.25),
    );
    wall(scene, Vec3::new(-0.6, 0.0, -3.9), Vec3::new(1.6, 0.8, -2.9));
    wall(
        scene,
        Vec3::new(-0.7, 0.8, -4.0),
        Vec3::new(1.7, 0.95, -2.8),
    );
    kit::prop(scene, "column").place(scene, Vec3::new(-1.6, 0.0, -2.2), 0.4, h);
    // Between the lamp and the column, a barrel (once a red post).
    kit::prop(scene, "winebarrel").place(scene, Vec3::new(-0.2, 0.0, -1.4), 0.7, 1.05);
    let crate_ = kit::prop(scene, "crate");
    crate_.place(scene, Vec3::new(3.2, 0.0, -3.2), 0.2, 0.6);
    crate_.place(scene, Vec3::new(3.25, 0.6, -3.15), -0.1, 0.6);
    crate_.place(scene, Vec3::new(3.3, 0.0, 3.2), 0.5, 0.6);
}

/// A roofed stone room (a crypt, [`crypt_room`]) on a lit field: a low
/// sun outside reaches the floor only through the door, and a shadowed
/// point light hanging inside in a glowing glass box that casts no shadow
/// (`Instance::cast_shadows` false: The Kestrel Stair's crypt lamp, web
/// findings #4), so the light reaches the altar and the column and throws
/// the barrel's shadow beside them. The camera stands inside by the door,
/// looking west at the altar.
pub fn crypt_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    crypt_room(&mut scene);
    // A low sun from the east: outside it lights the field, inside only
    // the patch the door lets in; the roof and the walls shadow the rest.
    let sun = Vec3::new(0.75, 0.42, 0.3).normalize();
    scene.lighting.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 4.0,
        max_shadow_distance: 70.0,
        ..Default::default()
    };
    scene.lighting.environment.sky.sun_direction = sun;
    scene.lighting.environment.intensity = 0.12;
    // The lamp hangs on a chain from the roof, in a glass lantern lit by
    // its flame.
    let lamp = Vec3::new(0.4, 2.2, 0.6);
    let glass = scene.add_material(Material {
        base_color: [1.0, 0.8, 0.4, 1.0],
        emissive: [0.45, 0.26, 0.1],
        ..Default::default()
    });
    let iron = scene.add_material(Material {
        base_color: [0.05, 0.05, 0.05, 1.0],
        metallic: 1.0,
        roughness: 0.5,
        ..Default::default()
    });
    let chain = scene.add_mesh(shapes::cuboid(Vec3::new(0.03, 4.0 - 2.35, 0.03)));
    scene.add_instance(
        chain,
        iron,
        Mat4::from_translation(lamp + Vec3::Y * (0.15 + (4.0 - 2.35) * 0.5)),
    );
    let lantern = scene.add_mesh(shapes::cuboid(Vec3::new(0.16, 0.24, 0.16)));
    scene.objects.instances.push(og_render::Instance {
        mesh: lantern,
        material: glass,
        transform: Mat4::from_translation(lamp),
        material_layer: None,
        fx: og_render::InstanceFx::NONE,
        cast_shadows: false,
        receive_shadows: true,
        sway: 0.0,
    });
    scene.lighting.lights = vec![Light {
        position: lamp,
        color: Vec3::new(1.0, 0.62, 0.3),
        intensity: 22.0,
        range: 9.0,
        cast_shadows: true,
        ..Default::default()
    }];
    scene.frame.camera = Camera {
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
    /// Behind the wall, in its shadow: a crate and a trickster that
    /// receive, and a crate and a trickster with `receive_shadows` off.
    pub box_receives: std::ops::Range<usize>,
    pub model_receives: std::ops::Range<usize>,
    pub box_no_receive: std::ops::Range<usize>,
    pub model_no_receive: std::ops::Range<usize>,
}

/// Shadow flags per instance (`Instance::cast_shadows` and
/// `receive_shadows`; og.Mesh's and og.Model's castShadow and
/// receiveShadow): a troupe of tricksters performing in the old town's
/// square under a low sun from the east. In front, on the open setts, two
/// of them: the left one added as a model is (it casts) and the right one
/// with `cast_shadows` off (no shadow beside it). Behind them a 5 m rubble
/// wall on the east side throws its shadow over a row of four: a crate and
/// a trickster that receive (dark in the wall's shadow), then a crate and a
/// trickster with `receive_shadows` off, lit as if the wall were not there
/// while still casting on the ground.
pub fn shadow_flags_scene() -> Result<FlagsScene, String> {
    let mut scene = RenderScene::new();
    crate::lighting_scenes::market_square(&mut scene);
    let model = load("trickster.glb")?;
    let parts = model.add_to(&mut scene);
    let scale = 1.8 / (model.max - model.min).max_element().max(1e-3);
    let trickster = |scene: &mut RenderScene, x: f32, z: f32, yaw: f32| {
        let first = scene.objects.instances.len();
        parts.instance(
            scene,
            Mat4::from_translation(Vec3::new(x, -model.min.y * scale, z))
                * Mat4::from_rotation_y(crate::scenes::TRICKSTER_TO_Z + yaw)
                * Mat4::from_scale(Vec3::splat(scale)),
        );
        first..scene.objects.instances.len()
    };
    let model_as_is = trickster(&mut scene, -1.6, 2.2, 0.4);
    let model_no_cast = trickster(&mut scene, 2.4, 2.2, -0.4);
    let wall = scene.objects.instances.len();
    let stone = crate::kit::surface_tinted(&mut scene, "stone", [0.95, 0.9, 0.84]);
    let slab = scene.add_mesh(crate::kit::block(
        Vec3::new(3.35, 0.0, -6.2),
        Vec3::new(3.85, 5.0, -0.2),
        2.4,
    ));
    scene.add_instance(slab, stone, Mat4::IDENTITY);
    let crate_ = crate::kit::prop(&mut scene, "crate");
    let block = |scene: &mut RenderScene, x: f32| {
        let first = scene.objects.instances.len();
        crate_.place(scene, Vec3::new(x, 0.0, -3.0), 0.35, 0.9);
        first..scene.objects.instances.len()
    };
    let box_receives = block(&mut scene, 2.3);
    let model_receives = trickster(&mut scene, 1.0, -3.0, 0.2);
    let box_no_receive = block(&mut scene, -0.4);
    let model_no_receive = trickster(&mut scene, -1.8, -3.0, 0.2);
    for i in model_no_cast.clone() {
        scene.objects.instances[i].cast_shadows = false;
    }
    for i in model_no_receive.clone().chain(box_no_receive.clone()) {
        scene.objects.instances[i].receive_shadows = false;
    }
    let sun = Vec3::new(0.85, 0.42, 0.12).normalize();
    crate::sky_scenes::physical_light_under(&mut scene, sun);
    scene.lighting.light.max_shadow_distance = 60.0;
    scene.frame.camera = Camera {
        position: Vec3::new(-1.6, 2.3, 9.8),
        target: Vec3::new(0.8, 1.6, -1.6),
        far: 2500.0,
        exposure: Exposure::Manual { ev: -0.3 },
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

/// Where [`local_contact_scene`]'s box stands and how large it is.
pub const CONTACT_BOX: Vec3 = Vec3::new(0.8, 0.8, 0.8);

/// The camera of [`local_contact_scene`].
pub fn local_contact_camera() -> Camera {
    Camera {
        position: Vec3::new(0.6, 2.6, 4.2),
        target: Vec3::new(0.5, 0.2, 0.0),
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    }
}

/// A local light's contact shadows, as in a Cornell box: a 0.8 m box at
/// the origin and a 0.5 m ball beside it on a plain light-grey floor
/// before a wall, lit only by one shadowed light up and to the left
/// (a spot aimed at the box, or a point light with `point`). The box's
/// shadow falls to its right and should start at its base, with no lit
/// line under it, and its edges should be soft and straight.
pub fn local_contact_scene(point: bool, shadows: bool) -> RenderScene {
    let mut scene = contact_room(point, shadows);
    let block = scene.add_mesh(shapes::cuboid(CONTACT_BOX));
    let white = contact_white(&mut scene);
    scene.add_instance(
        block,
        white,
        Mat4::from_translation(Vec3::new(0.0, CONTACT_BOX.y * 0.5, 0.0)),
    );
    let ball = scene.add_mesh(shapes::sphere(48, 32));
    scene.add_instance(
        ball,
        white,
        Mat4::from_translation(Vec3::new(-1.0, 0.25, 0.9)) * Mat4::from_scale(Vec3::splat(0.5)),
    );
    scene
}

/// [`local_contact_scene`]'s room and light with nothing standing on the
/// floor but a second floor under it: the same plane facing down, in the
/// floor's own plane, both single-sided (closed, as window-room.glb's
/// floor of two coplanar quads facing apart). The floor must stay lit.
pub fn twin_floor_scene(point: bool, shadows: bool) -> RenderScene {
    let mut scene = contact_room(point, shadows);
    let under = scene.add_mesh(shapes::plane(12.0, 1.0));
    let grey = scene.objects.instances[0].material;
    scene.add_instance(under, grey, Mat4::from_rotation_x(std::f32::consts::PI));
    scene
}

/// [`contact_silhouettes_scene`]'s ball (centre, radius), resting on the
/// floor.
pub const SILHOUETTE_BALL: (Vec3, f32) = (Vec3::new(1.1, 0.4, -0.6), 0.4);

/// [`contact_silhouettes_scene`]'s pole (its box's least and greatest
/// corners): 8 cm thick and 1.2 m tall, standing on the floor.
pub const SILHOUETTE_POLE: (Vec3, Vec3) =
    (Vec3::new(-0.54, 0.0, -0.64), Vec3::new(-0.46, 1.2, -0.56));

/// [`local_contact_scene`]'s room, light and camera with a large ball and a
/// thin pole in place of the box: from the camera, much of the lit floor
/// and wall lies just past their silhouettes, at a depth gap behind them,
/// where a screen-space trace towards the light passes behind them
/// without touching them. Contact shadows must leave it lit.
pub fn contact_silhouettes_scene(point: bool) -> RenderScene {
    let mut scene = contact_room(point, true);
    let white = contact_white(&mut scene);
    let (lo, hi) = SILHOUETTE_POLE;
    let pole = scene.add_mesh(shapes::cuboid(hi - lo));
    scene.add_instance(pole, white, Mat4::from_translation((lo + hi) * 0.5));
    let (centre, radius) = SILHOUETTE_BALL;
    let ball = scene.add_mesh(shapes::sphere(48, 32));
    scene.add_instance(
        ball,
        white,
        // shapes::sphere has radius 0.5.
        Mat4::from_translation(centre) * Mat4::from_scale(Vec3::splat(2.0 * radius)),
    );
    scene
}

/// The contact scenes' light, both their caster's position and the
/// contact tests' (shadows.rs).
pub const CONTACT_LIGHT: Vec3 = Vec3::new(-1.6, 3.0, 0.4);

fn contact_white(scene: &mut RenderScene) -> og_render::MaterialHandle {
    scene.add_material(Material {
        base_color: [0.85, 0.85, 0.85, 1.0],
        roughness: 0.7,
        ..Default::default()
    })
}

/// The contact scenes' floor and wall, their one light and their camera.
fn contact_room(point: bool, shadows: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    let grey = scene.add_material(Material {
        base_color: [0.75, 0.75, 0.75, 1.0],
        roughness: 0.9,
        ..Default::default()
    });
    let floor = scene.add_mesh(shapes::plane(12.0, 1.0));
    scene.add_instance(floor, grey, Mat4::IDENTITY);
    let wall = scene.add_mesh(shapes::plane_xy(12.0, 6.0));
    scene.add_instance(
        wall,
        grey,
        Mat4::from_translation(Vec3::new(0.0, 3.0, -2.0)),
    );
    let position = CONTACT_LIGHT;
    scene.lighting.lights.push(Light {
        kind: if point {
            LightKind::Point
        } else {
            LightKind::Spot {
                direction: (Vec3::new(0.4, 0.0, 0.0) - position).normalize(),
                angle_degrees: 55.0,
                penumbra: 0.2,
            }
        },
        position,
        color: Vec3::new(1.0, 0.95, 0.88),
        intensity: 30.0,
        range: 12.0,
        cast_shadows: shadows,
    });
    scene.lighting.light = DirectionalLight {
        intensity: 0.0,
        cast_shadows: false,
        ..Default::default()
    };
    scene.lighting.environment = Environment {
        intensity: 0.05,
        background: Background::Sky,
        ..Default::default()
    };
    scene.frame.camera = local_contact_camera();
    scene
}
