//! Shadow and antialiasing scenes of the shadows-aa round
//! (`crates/og-render/README.md` §Shadows and antialiasing): the
//! sun's soft shadows with cascade blending, eight shadowed local lights,
//! and the thin geometry under TAA with motion, Catmull-Rom history and
//! sharpening.

use glam::{Mat4, Vec3};
use og_render::{
    Background, Camera, DirectionalLight, Environment, Exposure, LightDesc, LightKind, Material,
    MaterialHandle, RenderScene, shapes,
};

/// The sun's PCSS softness and cascade blend of `shadows-soft`.
pub const SOFTNESS: f32 = 0.03;
pub const CASCADE_BLEND: f32 = 0.2;

/// A lawn 520 m across under hills all round (low towards `open`, a
/// bearing in xz, for a low sun to stand on the horizon), the ground the
/// lighting and shadows scenes stand on.
pub(crate) fn meadow(scene: &mut RenderScene, open: Option<glam::Vec2>) {
    use crate::kit;
    use glam::Vec2;
    let grass = kit::surface_tinted(scene, "grass", [0.82, 0.95, 0.66]);
    let lawn = scene.add_mesh(kit::slab(Vec2::splat(-260.0), Vec2::splat(260.0), 0.0, 7.0));
    scene.add_instance(lawn, grass, Mat4::IDENTITY);
    kit::hills(scene, open);
}

/// The shrine court the lighting and shadows scenes stand in: pale stone
/// flags 32 m wide inside a low rubble wall at its back (z = -9), a lawn
/// beyond running out to hills all round. Lit by nothing: the scenes light
/// it. Returns the wall's stone, for what else a scene builds of it.
pub(crate) fn court(scene: &mut RenderScene) -> MaterialHandle {
    use crate::kit;
    use glam::Vec2;
    meadow(scene, None);
    let flags = kit::surface_tinted(scene, "stone", [1.05, 0.98, 0.88]);
    let court = scene.add_mesh(kit::slab(
        Vec2::new(-16.0, -9.0),
        Vec2::new(16.0, 30.0),
        0.01,
        2.5,
    ));
    scene.add_instance(court, flags, Mat4::IDENTITY);
    let rubble = kit::surface_tinted(scene, "stone", [0.75, 0.7, 0.62]);
    let wall = scene.add_mesh(kit::block(
        Vec3::new(-16.0, 0.0, -9.6),
        Vec3::new(16.0, 1.3, -9.0),
        1.5,
    ));
    scene.add_instance(wall, rubble, Mat4::IDENTITY);
    rubble
}

/// How high a stone lantern's fire box sits, as a share of its height.
const TORO_FIRE: f32 = 0.66;

/// Stands the kit's stone lantern (`toro`, added once by the caller) on
/// `foot`, `height` metres tall, and returns the middle of its fire box.
pub(crate) fn toro(
    scene: &mut RenderScene,
    toro: &crate::kit::Prop,
    foot: Vec3,
    yaw: f32,
    height: f32,
) -> Vec3 {
    toro.place(scene, foot, yaw, height);
    foot + Vec3::Y * (height * TORO_FIRE)
}

/// Lights a stone lantern's fire box at `fire`: a flame glowing in it
/// (casting nothing) and a warm point light of `intensity` there.
pub(crate) fn light_toro(scene: &mut RenderScene, fire: Vec3, intensity: f32, shadowed: bool) {
    let flame = scene.add_mesh(shapes::sphere(12, 8));
    let glow = scene.add_standard_material(og_render::StandardMaterial {
        color: Vec3::ZERO,
        emissive: Vec3::new(1.0, 0.62, 0.3),
        emissiveIntensity: 3.0,
        ..Default::default()
    });
    scene.objects.instances.push(og_render::Instance {
        mesh: flame,
        material: glow,
        transform: Mat4::from_scale_rotation_translation(
            Vec3::new(0.14, 0.2, 0.14),
            glam::Quat::IDENTITY,
            fire,
        ),
        material_layer: None,
        fx: og_render::InstanceFx::NONE,
        cast_shadows: false,
        receive_shadows: false,
        sway: 0.0,
    });
    scene.lighting.lights.push(LightDesc {
        position: fire,
        color: Vec3::new(1.0, 0.62, 0.32),
        intensity,
        range: 14.0,
        cast_shadows: shadowed,
        ..Default::default()
    });
}

/// `shadows-soft`: the shrine court at golden hour, a vermilion torii (a
/// Meshy model) across it, a granite bench and a stone ball on its plinth
/// beside it, the sun 6° up behind the gate. The camera sits low and looks
/// back along the posts' long shadows: sharp where each post stands,
/// widening and softening towards the lens (the PCSS penumbra growing with
/// the distance to the caster), the lintel's shadow softest of all; the
/// bench's legs keep tight contact shadows. `soft` turns PCSS and the
/// cascade blend on.
pub fn soft_shadows_scene(soft: bool) -> RenderScene {
    use crate::kit;
    let mut scene = RenderScene::new();
    let rubble = court(&mut scene);
    // The gate across the path, 4.2 m high.
    kit::prop(&mut scene, "torii").place(&mut scene, Vec3::new(0.0, 0.0, -2.0), 0.0, 4.2);
    kit::prop(&mut scene, "stone_bench").place(&mut scene, Vec3::new(-4.6, 0.0, 0.6), 0.25, 0.48);
    let stone = scene.add_material(Material {
        base_color: [0.5, 0.45, 0.38, 1.0],
        roughness: 0.8,
        ..Default::default()
    });
    let plinth = scene.add_mesh(kit::block(
        Vec3::new(2.9, 0.0, -0.4),
        Vec3::new(4.1, 0.45, 0.8),
        1.0,
    ));
    scene.add_instance(plinth, rubble, Mat4::IDENTITY);
    let ball = scene.add_mesh(shapes::sphere(48, 32));
    // shapes::sphere is 1 m across: a ball 1 m across on the plinth.
    scene.add_instance(
        ball,
        stone,
        Mat4::from_translation(Vec3::new(3.5, 0.95, 0.2)),
    );
    // Golden hour: the sun 6° up, behind the gate's right.
    let sun = Vec3::new(0.55, 0.105, -0.83).normalize();
    crate::sky_scenes::physical_light_under(&mut scene, sun);
    let light = &mut scene.lighting.light;
    light.cast_shadows = true;
    light.max_shadow_distance = 90.0;
    light.shadow_softness = if soft { SOFTNESS } else { 0.0 };
    light.cascade_blend = if soft { CASCADE_BLEND } else { 0.0 };
    // Low beside the left post's shadow, looking back along it to the gate.
    let away = Vec3::new(-sun.x, 0.0, -sun.z).normalize();
    let side = Vec3::new(away.z, 0.0, -away.x);
    let post = Vec3::new(-2.0, 0.0, -2.0);
    scene.frame.camera = Camera {
        position: post + away * 11.0 + side * 2.2 + Vec3::Y * 0.9,
        target: post + away * 1.0 + side * 0.6 + Vec3::Y * 1.9,
        far: 2500.0,
        exposure: Exposure::Manual { ev: -0.2 },
        ..Default::default()
    };
    scene
}

/// `shadows-local-8`: the shrine court at night, lit by eight shadowed
/// lights: six stone lanterns in a ring round the torii, the bench and the
/// stone ball (each a point light in its fire box), and two spots hung
/// under the gate's tie beam, aimed down at the flags before it; with
/// `extra`, two more lanterns far back by the wall, whose shadows fall past
/// the tier's eight and are left out (the farthest and the faintest at the
/// camera). The gate's posts, the bench and the ball throw a star of
/// shadows, one from each light near it.
pub fn many_shadowed_lights(extra: bool) -> RenderScene {
    use crate::kit;
    let mut scene = RenderScene::new();
    let rubble = court(&mut scene);
    crate::lighting_scenes::night(&mut scene);
    kit::prop(&mut scene, "torii").place(&mut scene, Vec3::new(0.0, 0.0, -2.0), 0.0, 4.2);
    kit::prop(&mut scene, "stone_bench").place(&mut scene, Vec3::new(-2.6, 0.0, 0.6), 0.3, 0.48);
    let plinth = scene.add_mesh(crate::kit::block(
        Vec3::new(1.9, 0.0, -0.9),
        Vec3::new(2.9, 0.4, 0.1),
        1.0,
    ));
    scene.add_instance(plinth, rubble, Mat4::IDENTITY);
    let stone = scene.add_material(Material {
        base_color: [0.42, 0.4, 0.37, 1.0],
        roughness: 0.75,
        ..Default::default()
    });
    let ball = scene.add_mesh(shapes::sphere(48, 32));
    scene.add_instance(
        ball,
        stone,
        Mat4::from_scale_rotation_translation(
            Vec3::splat(0.8),
            glam::Quat::IDENTITY,
            Vec3::new(2.4, 0.8, -0.4),
        ),
    );
    let lantern = kit::prop(&mut scene, "toro");
    let mut ring: Vec<Vec3> = (0..6)
        .map(|k| {
            let a = std::f32::consts::TAU * (k as f32 + 0.5) / 6.0;
            Vec3::new(6.6 * a.sin(), 0.0, -0.6 + 5.4 * a.cos())
        })
        .collect();
    if extra {
        ring.extend([Vec3::new(-11.0, 0.0, -7.6), Vec3::new(11.5, 0.0, -7.8)]);
    }
    let first = scene.objects.instances.len();
    let fires: Vec<Vec3> = ring
        .iter()
        .map(|&foot| toro(&mut scene, &lantern, foot, (-foot.x).atan2(-foot.z), 1.7))
        .collect();
    // Each light hangs inside its own lantern's fire box, whose roof and
    // base would shade it into a ring of black wedges: the lanterns cast
    // nothing, and the gate, the bench and the ball throw the shadows.
    for i in &mut scene.objects.instances[first..] {
        i.cast_shadows = false;
    }
    for (i, fire) in fires.iter().enumerate() {
        light_toro(&mut scene, *fire, if i < 6 { 26.0 } else { 16.0 }, true);
    }
    // A wide ring and short reaches, so each lantern's pool stands apart
    // and every caster's shadow runs out of it as one long dark spoke
    // instead of every light filling the others' shadows.
    for l in &mut scene.lighting.lights {
        l.range = 8.0;
    }
    // The two spots under the tie beam (inserted among the lanterns: the
    // nearest and brightest lights keep their shadows).
    for (k, x) in [-1.1f32, 1.1].into_iter().enumerate() {
        scene.lighting.lights.insert(
            3 + k,
            LightDesc {
                kind: LightKind::Spot {
                    direction: Vec3::new(0.0, -1.0, 0.45).normalize(),
                    angle_degrees: 46.0,
                    penumbra: 0.4,
                },
                position: Vec3::new(x, 3.2, -1.85),
                color: Vec3::new(1.0, 0.85, 0.66),
                intensity: 60.0,
                range: 9.0,
                radius: 0.0,
                cast_shadows: true,
                mobility: og_render::LightMobility::Movable,
            },
        );
    }
    scene.frame.camera = Camera {
        position: Vec3::new(0.4, 4.2, 10.0),
        target: Vec3::new(0.0, 0.3, -1.2),
        far: 2500.0,
        exposure: Exposure::Manual { ev: -0.4 },
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
    scene.lighting.lights = vec![LightDesc {
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
    /// The front lantern as it is (casting).
    pub model_as_is: std::ops::Range<usize>,
    /// The front lantern with `cast_shadows` off.
    pub model_no_cast: std::ops::Range<usize>,
    /// The wall.
    pub wall: usize,
    /// Behind the wall, in its shadow: a block and a lantern that
    /// receive, and a block and a lantern with `receive_shadows` off.
    pub box_receives: std::ops::Range<usize>,
    pub model_receives: std::ops::Range<usize>,
    pub box_no_receive: std::ops::Range<usize>,
    pub model_no_receive: std::ops::Range<usize>,
}

/// Shadow flags per instance (`Instance::cast_shadows` and
/// `receive_shadows`; og.Mesh's and og.Model's castShadow and
/// receiveShadow), in the shrine court under a golden sun from the east.
/// In front, on the open flags, two stone lanterns (models): the left one
/// as it is (it casts) and the right one with `cast_shadows` off (no shadow
/// beside it). Behind them a 3.2 m coped rubble wall on the east side
/// throws its shadow over a row of four: a stone block and a lantern that
/// receive (dark in the wall's shadow), then a block and a lantern with
/// `receive_shadows` off, lit as if the wall were not there while still
/// casting on the flags.
pub fn shadow_flags_scene() -> Result<FlagsScene, String> {
    let mut scene = RenderScene::new();
    let rubble = court(&mut scene);
    let lantern = crate::kit::prop(&mut scene, "toro");
    let model = |scene: &mut RenderScene, x: f32, z: f32, yaw: f32| {
        let first = scene.objects.instances.len();
        lantern.place(scene, Vec3::new(x, 0.0, z), yaw, 1.9);
        first..scene.objects.instances.len()
    };
    let model_as_is = model(&mut scene, -2.8, 1.4, 0.4);
    let model_no_cast = model(&mut scene, 2.4, 1.6, -0.4);
    let wall = scene.objects.instances.len();
    let slab = scene.add_mesh(crate::kit::block(
        Vec3::new(3.35, 0.0, -6.2),
        Vec3::new(3.85, 3.2, -0.2),
        2.4,
    ));
    scene.add_instance(slab, rubble, Mat4::IDENTITY);
    // Its coping: dark weathered planks overhanging both faces.
    let planks = crate::kit::surface_tinted(&mut scene, "planks", [0.42, 0.36, 0.3]);
    let coping = scene.add_mesh(crate::kit::block(
        Vec3::new(3.2, 3.2, -6.35),
        Vec3::new(4.0, 3.38, -0.05),
        1.2,
    ));
    scene.add_instance(coping, planks, Mat4::IDENTITY);
    let granite = crate::kit::surface_tinted(&mut scene, "stone", [0.62, 0.6, 0.58]);
    let cube = scene.add_mesh(crate::kit::block(
        Vec3::new(-0.45, 0.0, -0.45),
        Vec3::new(0.45, 0.9, 0.45),
        1.0,
    ));
    let block = |scene: &mut RenderScene, x: f32| {
        let first = scene.objects.instances.len();
        scene.add_instance(
            cube,
            granite,
            Mat4::from_translation(Vec3::new(x, 0.0, -3.0)) * Mat4::from_rotation_y(0.35),
        );
        first..scene.objects.instances.len()
    };
    let box_receives = block(&mut scene, 2.3);
    let model_receives = model(&mut scene, 1.0, -3.0, 0.2);
    let box_no_receive = block(&mut scene, -0.4);
    let model_no_receive = model(&mut scene, -1.8, -3.0, 0.2);
    for i in model_no_cast.clone() {
        scene.objects.instances[i].cast_shadows = false;
    }
    for i in model_no_receive.clone().chain(box_no_receive.clone()) {
        scene.objects.instances[i].receive_shadows = false;
    }
    // A golden sun 9° up: the 3.2 m wall's shadow still covers the row
    // 5 m behind it to the lanterns' tops.
    let sun = Vec3::new(0.85, 0.15, 0.3).normalize();
    crate::sky_scenes::physical_light_under(&mut scene, sun);
    scene.lighting.light.max_shadow_distance = 60.0;
    scene.frame.camera = Camera {
        position: Vec3::new(-0.6, 1.6, 6.2),
        target: Vec3::new(0.3, 0.8, -3.0),
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
    scene.lighting.lights.push(LightDesc {
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
        radius: 0.0,
        cast_shadows: shadows,
        mobility: og_render::LightMobility::Movable,
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
