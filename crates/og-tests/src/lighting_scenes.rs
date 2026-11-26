//! The lighting and shadows scenes, each a simple stage for the feature it
//! is named for: an avenue of torii, the shrine court
//! (`shadow_scenes::court`) at noon, dusk and night, a gallery, a loggia
//! and the ruined colonnade, with only the few things the feature acts
//! on, lit with care. The old town's street and square
//! ([`high_street`], `market_square`) stay for the scenes of other
//! families. Where a perf or pixel test measures a fixed workload it keeps
//! its own plain fixture (`scenes::cascades_200m`, `scenes::dim_scene`,
//! `scenes::point_spot_lights`, `p2_scenes::many_lights`,
//! `shadow_scenes::local_contact_scene`, `fork_scenes::shadow_box_scene`
//! and `hemisphere_scene`); the shots here are the registered scenes.

use crate::kit;
use crate::sky_scenes::physical_light_under;
use glam::{Mat4, Quat, Vec2, Vec3};
use og_render::{Camera, Exposure, LensFlare, LightDesc, LightKind, RenderScene, shapes};

/// A street lamp's lantern: where its glass glows and its light hangs.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Lantern {
    pub glass: Vec3,
    /// The kerb's side (-1 left, +1 right).
    pub side: f32,
}

/// The old town's high street: setts 8 m wide between kerbed pavements,
/// townhouses shoulder to shoulder on both sides from z = 8 to `len`
/// metres down the street, an iron street lamp every 14 m on each kerb, the
/// watchtower closing the view past its end and hills all round. Returns
/// the lamps' lanterns, nearest first (unlit: a scene lights those it
/// wants).
pub(crate) fn high_street(scene: &mut RenderScene, len: f32) -> Vec<Lantern> {
    let setts = kit::surface_tinted(scene, "cobbles", [0.86, 0.84, 0.8]);
    let road = scene.add_mesh(kit::slab(
        Vec2::new(-4.0, -len - 40.0),
        Vec2::new(4.0, 14.0),
        0.0,
        2.0,
    ));
    scene.add_instance(road, setts, Mat4::IDENTITY);
    // The square past the street's end, of the same setts, and grass beyond.
    let square = scene.add_mesh(kit::slab(
        Vec2::new(-40.0, -len - 90.0),
        Vec2::new(40.0, -len - 40.0),
        0.0,
        2.0,
    ));
    scene.add_instance(square, setts, Mat4::IDENTITY);
    let flags = kit::surface_tinted(scene, "cobbles", [0.98, 0.95, 0.9]);
    let grass = kit::surface_tinted(scene, "grass", [0.86, 1.0, 0.72]);
    for (min, max) in [
        (Vec2::new(-260.0, -260.0), Vec2::new(-40.0, 260.0)),
        (Vec2::new(40.0, -260.0), Vec2::new(260.0, 260.0)),
        (Vec2::new(-40.0, -260.0), Vec2::new(40.0, -len - 90.0)),
        (Vec2::new(-40.0, 14.0), Vec2::new(40.0, 260.0)),
    ] {
        let mesh = scene.add_mesh(kit::slab(min, max, 0.0, 7.0));
        scene.add_instance(mesh, grass, Mat4::IDENTITY);
    }
    kit::hills(scene, None);
    let house = kit::prop(scene, "townhouse");
    let lamp = kit::prop(scene, "streetlamp");
    let mut lanterns = Vec::new();
    let houses = (len / 7.8).ceil() as i32 + 1;
    for side in [-1.0f32, 1.0] {
        // The pavement and its kerb, 15 cm up, of larger setts, running on
        // under the houses (their fronts stand on it).
        let pavement = scene.add_mesh(kit::block(
            Vec3::new(if side < 0.0 { -14.0 } else { 4.0 }, 0.0, -len - 40.0),
            Vec3::new(if side < 0.0 { -4.0 } else { 14.0 }, 0.15, 14.0),
            3.2,
        ));
        scene.add_instance(pavement, flags, Mat4::IDENTITY);
        // The houses' fronts on the pavement's back edge, facing the
        // street, each a little taller or lower than its neighbour; the
        // row stops short of the square.
        for k in 0..houses {
            let z = 6.0 - 7.8 * k as f32 + if side > 0.0 { 3.4 } else { 0.0 };
            if z < -len - 36.0 {
                break;
            }
            let height = 11.5 + 2.5 * ((k as f32 * 2.3 + side).sin() * 0.5 + 0.5);
            house.place(
                scene,
                Vec3::new(side * 8.75, 0.15, z),
                -side * std::f32::consts::FRAC_PI_2,
                height,
            );
        }
        for k in 0..((len / 14.0) as i32 + 1) {
            let z = -2.0 - 14.0 * k as f32 + if side > 0.0 { 7.0 } else { 0.0 };
            let foot = Vec3::new(side * 4.4, 0.15, z);
            lamp.place(scene, foot, 0.0, 4.2);
            lanterns.push(Lantern {
                glass: foot + Vec3::Y * 3.35,
                side,
            });
        }
    }
    lanterns.sort_by(|a, b| b.glass.z.total_cmp(&a.glass.z));
    // The watchtower over the square at the street's end.
    kit::prop(scene, "watchtower").place(scene, Vec3::new(9.0, 0.0, -len - 62.0), 2.6, 26.0);
    lanterns
}

/// Lights `lanterns`: a warm glowing bulb in each, and a point light of
/// `intensity` beside it (shadowed for the first `shadowed`), the glass
/// glowing at `glass`.
pub(crate) fn light_lanterns(
    scene: &mut RenderScene,
    lanterns: &[Lantern],
    intensity: f32,
    shadowed: usize,
    glass: f32,
) {
    let bulb = scene.add_mesh(shapes::sphere(12, 8));
    let glow = scene.add_standard_material(og_render::StandardMaterial {
        color: Vec3::ZERO,
        emissive: Vec3::new(1.0, 0.72, 0.45),
        emissiveIntensity: glass,
        ..Default::default()
    });
    for (i, l) in lanterns.iter().enumerate() {
        scene.add_instance(
            bulb,
            glow,
            Mat4::from_scale_rotation_translation(Vec3::splat(0.16), Quat::IDENTITY, l.glass),
        );
        scene.lighting.lights.push(LightDesc {
            kind: LightKind::Point,
            position: l.glass - Vec3::new(l.side * 0.3, 0.25, 0.0),
            color: Vec3::new(1.0, 0.72, 0.45),
            intensity,
            range: 16.0,
            radius: 0.0,
            cast_shadows: i < shadowed,
            mobility: og_render::LightMobility::Movable,
        });
    }
}

/// The market's wares at a spot of the street: crates stacked, barrels,
/// and a counter of produce, `turn` about +y.
pub(crate) fn wares(scene: &mut RenderScene, at: Vec3, turn: f32) {
    let r = Mat4::from_translation(at) * Mat4::from_rotation_y(turn);
    let crate_ = kit::prop(scene, "crate");
    let barrel = kit::prop(scene, "winebarrel");
    let counter = kit::prop(scene, "counter");
    let put = |scene: &mut RenderScene, p: &kit::Prop, x: f32, y: f32, z: f32, yaw: f32, h: f32| {
        p.place_by(scene, r * p.transform(Vec3::new(x, y, z), yaw, h));
    };
    put(scene, &counter, 0.0, 0.0, 0.0, 0.0, 1.35);
    put(scene, &crate_, -1.9, 0.0, 0.3, 0.3, 0.6);
    put(scene, &crate_, -1.85, 0.6, 0.25, 0.05, 0.6);
    put(scene, &crate_, -2.6, 0.0, 0.9, -0.4, 0.6);
    put(scene, &barrel, 1.9, 0.0, 0.4, 0.0, 1.0);
    put(scene, &barrel, 2.5, 0.0, 1.1, 1.3, 1.0);
}

/// `cascades-200m`: an avenue of torii on a summer morning, a gate every
/// 6 m for 200 m up a flagged path through the lawn: the stone lanterns at
/// the camera's feet throw small sharp shadows in the first cascade, the
/// near gates' posts and lintels stripe the path in the middle ones, and
/// the far gates still shade it 190 m off in the last.
pub fn cascades_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    crate::shadow_scenes::meadow(&mut scene, None);
    let flags = kit::surface_tinted(&mut scene, "stone", [1.05, 0.98, 0.88]);
    let path = scene.add_mesh(kit::slab(
        Vec2::new(-2.8, -206.0),
        Vec2::new(2.8, 12.0),
        0.01,
        2.5,
    ));
    scene.add_instance(path, flags, Mat4::IDENTITY);
    let torii = kit::prop(&mut scene, "torii");
    for k in 0..34 {
        torii.place(&mut scene, Vec3::new(0.0, 0.0, -6.0 * k as f32), 0.0, 4.2);
    }
    let lantern = kit::prop(&mut scene, "toro");
    for (x, z) in [(-3.6f32, 4.0f32), (3.6, 4.0), (-3.6, -9.0), (3.6, -9.0)] {
        lantern.place(&mut scene, Vec3::new(x, 0.0, z), 0.0, 1.7);
    }
    // A morning sun ahead on the right: each gate's shadow falls back
    // across the path towards the camera's left, a diagonal stripe every
    // 6 m running on to the avenue's end.
    physical_light_under(&mut scene, Vec3::new(0.55, 0.4, -0.73));
    scene.lighting.light.max_shadow_distance = 220.0;
    scene.frame.camera = Camera {
        position: Vec3::new(1.2, 1.6, 9.5),
        target: Vec3::new(-0.3, 2.6, -60.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: -0.3 },
        ..Default::default()
    };
    scene
}

/// `exposure`: the shrine court at deep dusk, the sun gone below the
/// hills and two stone lanterns just lit by the torii: a view a fixed
/// exposure leaves near black, brought to a dusk's key by the metered one.
pub fn exposure_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    crate::shadow_scenes::court(&mut scene);
    kit::prop(&mut scene, "torii").place(&mut scene, Vec3::new(0.0, 0.0, -2.0), 0.0, 4.2);
    kit::prop(&mut scene, "stone_bench").place(&mut scene, Vec3::new(-4.2, 0.0, 0.4), 0.25, 0.48);
    let lantern = kit::prop(&mut scene, "toro");
    for x in [-3.2f32, 3.2] {
        let fire =
            crate::shadow_scenes::toro(&mut scene, &lantern, Vec3::new(x, 0.0, -0.6), 0.0, 1.7);
        crate::shadow_scenes::light_toro(&mut scene, fire, 8.0, false);
    }
    physical_light_under(&mut scene, Vec3::new(-0.7, -0.035, -0.71));
    // Metered a stop under its key.
    scene.frame.camera = Camera {
        position: Vec3::new(2.2, 1.2, 5.6),
        target: Vec3::new(-0.6, 1.9, -6.0),
        far: 2500.0,
        exposure: Exposure::Auto {
            compensation: -1.0,
            min_ev: -10.0,
            max_ev: 10.0,
            adapt_speed_up: 0.0,
            adapt_speed_down: 0.0,
        },
        ..Default::default()
    };
    scene
}

/// `flare-noon`: in the shrine court at noon, the camera low under the
/// torii looking up past its lintel at a sun almost overhead: its ghosts
/// and halo cross the sky beside the black kasagi.
pub fn flare_noon_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    crate::shadow_scenes::court(&mut scene);
    kit::prop(&mut scene, "torii").place(&mut scene, Vec3::new(0.0, 0.0, -2.0), 0.0, 4.2);
    let sun = Vec3::new(0.12, 1.0, -0.35).normalize();
    physical_light_under(&mut scene, sun);
    scene.look.post.lens_flare = Some(LensFlare::default());
    // Below the gate's right end, the lintel crossing the upper left of the
    // view and the sun standing in the sky to its right.
    let eye = Vec3::new(1.4, 1.2, 0.4);
    let lintel = (Vec3::new(-0.6, 4.0, -2.0) - eye).normalize();
    scene.frame.camera = Camera {
        position: eye,
        target: eye + (sun * 0.62 + lintel * 0.38).normalize(),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.4 },
        ..Default::default()
    };
    scene
}

/// How high the sun's line crosses the torii, as a share of its height:
/// a little under its lintel's top, which runs level across the middle.
const KASAGI_MIDDLE: f32 = 0.98;

/// `flare-edge`: a sun at `elevation` (the slope of its direction) low
/// over the hills, behind the top of a torii far out on a lawn, seen
/// through a long lens: the gate's black lintel hides a little more than
/// the lower half of the sun's disc, the camera turned a little
/// right of it, so the flare comes from the part of the disc in view.
pub fn flare_edge_scene(elevation: f32) -> RenderScene {
    let mut scene = RenderScene::new();
    let eye = Vec3::new(1.4, 1.2, 8.0);
    let along = Vec3::new(-0.04, 0.0, -1.0).normalize();
    crate::shadow_scenes::meadow(&mut scene, Some(Vec2::new(along.x, along.z)));
    let sun = (along + Vec3::Y * elevation).normalize();
    physical_light_under(&mut scene, sun);
    // The gate on the sun's line from the eye, as far off as puts its
    // lintel's top on that line.
    let h = 12.0;
    let d = (h * KASAGI_MIDDLE - eye.y) / elevation;
    // A long lens, so the gate and the sun's disc fill the view.
    let foot = eye + along * d;
    kit::prop(&mut scene, "torii").place(&mut scene, Vec3::new(foot.x, 0.0, foot.z), 0.0, h);
    scene.look.post.lens_flare = Some(LensFlare::default());
    let right = Vec3::new(-along.z, 0.0, along.x);
    scene.frame.camera = Camera {
        position: eye,
        target: eye + along + right * 0.05 + Vec3::Y * (elevation - 0.02),
        projection: og_render::Projection::Perspective {
            fov_y_degrees: 16.0,
        },
        far: 2500.0,
        exposure: Exposure::Manual { ev: -0.6 },
        ..Default::default()
    };
    scene
}

/// The sky brightness `god-rays` takes rays from.
pub const GOD_RAYS_THRESHOLD: f32 = 10.0;

/// `god-rays`: the ruined colonnade late in the day, the camera beside
/// the way looking obliquely down its left row into a sun low over the
/// arch: column after column cuts its glow into shafts that fan out
/// towards the viewer.
pub fn god_rays_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    crate::atmo_scenes::colonnade(&mut scene);
    let eye = Vec3::new(4.0, 1.7, 6.0);
    let along = (Vec3::new(-6.0, 0.0, -60.0) - eye).normalize();
    let sun = (along + Vec3::Y * 0.2).normalize();
    physical_light_under(&mut scene, sun);
    // The threshold is linear sky brightness before exposure: the physical
    // sky's blue already passes the default 2 (set for the plain sky), so
    // here only the sun's disc and glow carry rays.
    scene.look.post.god_rays = Some(og_render::GodRays {
        intensity: 0.8,
        threshold: GOD_RAYS_THRESHOLD,
        ..Default::default()
    });
    scene.frame.camera = Camera {
        position: eye,
        target: eye + along + Vec3::Y * 0.16,
        far: 2500.0,
        exposure: Exposure::Manual { ev: -1.6 },
        ..Default::default()
    };
    scene
}

/// The old town's market square: setts 70 m across, a row of townhouses
/// along its back (z = -27) and its right side (x = 30) facing in, the
/// left side open to a pine stand and the watchtower on the hills.
pub(crate) fn market_square(scene: &mut RenderScene) {
    let setts = kit::surface_tinted(scene, "cobbles", [0.86, 0.84, 0.8]);
    let (lo, hi) = (Vec2::new(-35.0, -32.0), Vec2::new(35.0, 22.0));
    let square = scene.add_mesh(kit::slab(lo, hi, 0.0, 2.0));
    scene.add_instance(square, setts, Mat4::IDENTITY);
    let grass = kit::surface_tinted(scene, "grass", [0.86, 1.0, 0.72]);
    for (min, max) in [
        (Vec2::new(-260.0, -260.0), Vec2::new(lo.x, 260.0)),
        (Vec2::new(hi.x, -260.0), Vec2::new(260.0, 260.0)),
        (Vec2::new(lo.x, -260.0), Vec2::new(hi.x, lo.y)),
        (Vec2::new(lo.x, hi.y), Vec2::new(hi.x, 260.0)),
    ] {
        let mesh = scene.add_mesh(kit::slab(min, max, 0.0, 7.0));
        scene.add_instance(mesh, grass, Mat4::IDENTITY);
    }
    kit::hills(scene, None);
    let house = kit::prop(scene, "townhouse");
    for k in 0..9 {
        let height = 11.5 + 2.5 * ((k as f32 * 2.3).sin() * 0.5 + 0.5);
        house.place(
            scene,
            Vec3::new(-31.0 + 7.8 * k as f32, 0.0, -28.5),
            0.0,
            height,
        );
    }
    for k in 0..6 {
        let height = 11.0 + 3.0 * ((k as f32 * 1.7 + 1.0).sin() * 0.5 + 0.5);
        house.place(
            scene,
            Vec3::new(31.5, 0.0, -18.0 + 7.8 * k as f32),
            -std::f32::consts::FRAC_PI_2,
            height,
        );
    }
    let pine = kit::prop(scene, "pine");
    for k in 0..40 {
        let h = |n: i32| crate::fx_scenes::hash2(k, n, 0x5a1);
        let at = Vec3::new(-48.0 - h(0) * 70.0, 0.0, 30.0 - h(1) * 110.0);
        pine.place(scene, at, h(2) * 6.0, 12.0 + h(3) * 8.0);
    }
    kit::prop(scene, "watchtower").place(scene, Vec3::new(-70.0, 0.0, -60.0), 0.7, 26.0);
}

/// Night over `scene`: a moonlit sky well after sunset, faint enough that
/// the local lights carry the shot.
pub(crate) fn night(scene: &mut RenderScene) {
    physical_light_under(scene, Vec3::new(-0.4, -0.25, -0.88));
}

/// The kit's car parked with its nose at `nose` (on the ground), facing
/// `heading` (xz), its headlights lit: two spots of `intensity` thrown
/// ahead and down, and glowing lamps where they are. Returns the
/// headlights' positions.
pub(crate) fn car_with_headlights(
    scene: &mut RenderScene,
    nose: Vec3,
    heading: Vec2,
    intensity: f32,
) -> [Vec3; 2] {
    let car = kit::prop(scene, "car");
    let ahead = Vec3::new(heading.x, 0.0, heading.y).normalize();
    let side = Vec3::new(-ahead.z, 0.0, ahead.x);
    let (length, height) = (3.1, 1.45);
    // The model's nose is its +x: turned so +x points along `heading`.
    let yaw = (-ahead.z).atan2(ahead.x);
    car.place(scene, nose - ahead * (length * 0.5), yaw, height);
    let lamp = scene.add_mesh(shapes::sphere(12, 8));
    let glow = scene.add_standard_material(og_render::StandardMaterial {
        color: Vec3::ZERO,
        emissive: Vec3::new(1.0, 0.95, 0.85),
        emissiveIntensity: 6.0,
        ..Default::default()
    });
    let lights = [-1.0f32, 1.0].map(|s| nose - ahead * 0.12 + side * (s * 0.55) + Vec3::Y * 0.66);
    for p in lights {
        scene.add_instance(
            lamp,
            glow,
            Mat4::from_scale_rotation_translation(Vec3::new(0.16, 0.1, 0.16), Quat::IDENTITY, p),
        );
        scene.lighting.lights.push(LightDesc {
            kind: LightKind::Spot {
                direction: (ahead - Vec3::Y * 0.12).normalize(),
                angle_degrees: 30.0,
                penumbra: 0.35,
            },
            position: p + ahead * 0.1,
            color: Vec3::new(1.0, 0.95, 0.85),
            intensity,
            range: 30.0,
            radius: 0.0,
            cast_shadows: false,
            mobility: og_render::LightMobility::Movable,
        });
    }
    lights
}

/// `flare-lights`: a shrine path at night between two rows of stone
/// lanterns, lit at different distances and flaring through their own
/// glow, the third on the left half hidden behind the post of a nearer
/// torii, and at the path's end a lamp on the last gate aimed down the
/// path into the lens (a spot).
pub fn flare_lights_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    crate::shadow_scenes::meadow(&mut scene, None);
    let flags = kit::surface_tinted(&mut scene, "stone", [1.05, 0.98, 0.88]);
    let path = scene.add_mesh(kit::slab(
        Vec2::new(-2.4, -60.0),
        Vec2::new(2.4, 10.0),
        0.01,
        2.5,
    ));
    scene.add_instance(path, flags, Mat4::IDENTITY);
    night(&mut scene);
    let torii = kit::prop(&mut scene, "torii");
    for z in [-3.0f32, -27.0, -45.0] {
        torii.place(&mut scene, Vec3::new(0.0, 0.0, z), 0.0, 4.2);
    }
    let lantern = kit::prop(&mut scene, "toro");
    let mut fires = Vec::new();
    for k in 0..6 {
        let z = 2.0 - 8.0 * k as f32;
        for x in [-3.2f32, 3.2] {
            fires.push(crate::shadow_scenes::toro(
                &mut scene,
                &lantern,
                Vec3::new(x, 0.0, z),
                0.0,
                1.7,
            ));
        }
    }
    for fire in &fires {
        crate::shadow_scenes::light_toro(&mut scene, *fire, 24.0, false);
    }
    // The lamp under the far gate's tie beam, aimed at the eye.
    let eye0 = Vec3::new(-1.0, 1.5, 9.0);
    let lamp = Vec3::new(0.0, 3.1, -45.0);
    let glow = scene.add_standard_material(og_render::StandardMaterial {
        color: Vec3::ZERO,
        emissive: Vec3::new(1.0, 0.92, 0.8),
        emissiveIntensity: 40.0,
        ..Default::default()
    });
    let bulb = scene.add_mesh(shapes::sphere(12, 8));
    scene.add_instance(
        bulb,
        glow,
        Mat4::from_scale_rotation_translation(Vec3::splat(0.22), Quat::IDENTITY, lamp),
    );
    scene.lighting.lights.push(LightDesc {
        kind: LightKind::Spot {
            direction: (eye0 - lamp).normalize(),
            angle_degrees: 28.0,
            penumbra: 0.4,
        },
        position: lamp,
        color: Vec3::new(1.0, 0.92, 0.8),
        intensity: 400.0,
        range: 70.0,
        radius: 0.0,
        cast_shadows: false,
        mobility: og_render::LightMobility::Movable,
    });
    // The eye slid sideways so the near gate's left post (x = -2.0 at
    // z = -3, 0.12 m in radius) hides the left half of the third left
    // lantern's fire.
    let (post, far) = (Vec3::new(-2.0, 0.0, -3.0), fires[4]);
    let mut eye = eye0;
    let t = (post.z - eye.z) / (far.z - eye.z);
    eye.x = (post.x + 0.12 - t * far.x) / (1.0 - t);
    scene.look.post.lens_flare = Some(LensFlare::default());
    scene.frame.camera = Camera {
        position: eye,
        target: Vec3::new(0.2, 2.0, -40.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: -0.4 },
        ..Default::default()
    };
    scene
}

/// `shadows-local-contact`: a dark gallery lit by one shadowed spot from
/// up and to the left, aimed at a granite block, a stone lantern and a
/// black ball on a polished floor: their shadows start at their bases
/// with no lit line under them, their edges soft and straight (the tent
/// PCF), and the ball's touches the floor at one point.
pub fn cellar_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    let floor = scene.add_standard_material(og_render::StandardMaterial {
        color: Vec3::splat(0.07),
        roughness: 0.3,
        ..Default::default()
    });
    let mesh = scene.add_mesh(kit::slab(
        Vec2::new(-10.0, -8.0),
        Vec2::new(10.0, 10.0),
        0.0,
        2.0,
    ));
    scene.add_instance(mesh, floor, Mat4::IDENTITY);
    let wall = kit::plaster(&mut scene, [0.16, 0.15, 0.14]);
    let mesh = scene.add_mesh(kit::block(
        Vec3::new(-8.0, 0.0, -3.4),
        Vec3::new(8.0, 5.0, -3.0),
        2.0,
    ));
    scene.add_instance(mesh, wall, Mat4::IDENTITY);
    let granite = kit::surface_tinted(&mut scene, "stone", [0.62, 0.6, 0.58]);
    let block = scene.add_mesh(kit::block(
        Vec3::new(-0.4, 0.0, -0.4),
        Vec3::new(0.4, 0.8, 0.4),
        1.0,
    ));
    scene.add_instance(
        block,
        granite,
        Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)) * Mat4::from_rotation_y(0.4),
    );
    kit::prop(&mut scene, "toro").place(&mut scene, Vec3::new(-1.15, 0.0, 0.35), 0.3, 1.5);
    let ball = scene.add_mesh(shapes::sphere(48, 32));
    let black = scene.add_standard_material(og_render::StandardMaterial {
        color: Vec3::splat(0.02),
        roughness: 0.15,
        ..Default::default()
    });
    scene.add_instance(
        ball,
        black,
        Mat4::from_scale_rotation_translation(
            Vec3::splat(0.6),
            Quat::IDENTITY,
            Vec3::new(1.15, 0.3, 0.6),
        ),
    );
    let position = Vec3::new(-2.0, 3.0, 1.6);
    scene.lighting.lights.push(LightDesc {
        kind: LightKind::Spot {
            direction: (Vec3::new(0.3, 0.0, -0.2) - position).normalize(),
            angle_degrees: 40.0,
            penumbra: 0.3,
        },
        position,
        color: Vec3::new(1.0, 0.84, 0.64),
        intensity: 90.0,
        range: 12.0,
        radius: 0.0,
        cast_shadows: true,
        mobility: og_render::LightMobility::Movable,
    });
    scene.lighting.light.intensity = 0.0;
    scene.lighting.light.cast_shadows = false;
    scene.lighting.environment = og_render::Environment {
        intensity: 0.03,
        background: og_render::Background::Color(Vec3::new(0.006, 0.007, 0.01)),
        ..Default::default()
    };
    scene.look.post.bloom = Some(og_render::Bloom::default());
    scene.frame.camera = Camera {
        position: Vec3::new(1.9, 1.15, 3.4),
        target: Vec3::new(-0.2, 0.45, -0.5),
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    };
    scene
}

/// A canopy of vine leaves: leaves scattered over a 256 px card, green in
/// their colour, alpha 0 between them (sRGB).
fn vine_leaves() -> og_render::TextureData {
    let n = 256u32;
    let mut rgba = vec![0u8; (n * n * 4) as usize];
    for k in 0..320 {
        let h = |s: i32| crate::fx_scenes::hash2(k, s, 0x71e);
        let (cx, cy) = (h(0) * n as f32, h(1) * n as f32);
        let r = 5.0 + h(2) * 7.0;
        let (a, shade) = (h(3) * std::f32::consts::TAU, 0.6 + 0.5 * h(4));
        let color = [0.22 * shade, 0.42 * shade, 0.12 * shade];
        let reach = r as i32 + 1;
        for dy in -reach..=reach {
            for dx in -reach..=reach {
                let (x, y) = (dx as f32, dy as f32);
                let (u, v) = (x * a.cos() + y * a.sin(), -x * a.sin() + y * a.cos());
                // A leaf: an ellipse with a lobed edge.
                let lobes = 1.0 + 0.18 * (5.0 * v.atan2(u)).cos();
                if (u * u / (r * r) + v * v / (0.7 * r * r)).sqrt() > lobes {
                    continue;
                }
                let px = (cx as i32 + dx).rem_euclid(n as i32) as u32;
                let py = (cy as i32 + dy).rem_euclid(n as i32) as u32;
                let i = ((py * n + px) * 4) as usize;
                let vein = if u.abs() < 0.6 { 0.8 } else { 1.0 };
                for c in 0..3 {
                    rgba[i + c] = ((color[c] * vein).powf(1.0 / 2.2) * 255.0) as u8;
                }
                rgba[i + 3] = 255;
            }
        }
    }
    og_render::TextureData {
        width: n,
        height: n,
        rgba,
        color_space: og_render::ColorSpace::Srgb,
        mips: og_render::Mips::Generate,
    }
}

/// `shadow-box`: a vine pergola in the shrine court on a summer
/// afternoon, drawn under one 8 m shadow box (the cheap tier's sun
/// shadows instead of cascades) focused 12 m ahead of the camera, on the
/// pergola: its posts, the table and benches under it, and the dappled
/// shade of its alpha-tested leaves are sharp, while the court beyond
/// the box goes unshadowed.
pub fn shadow_box_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    crate::shadow_scenes::court(&mut scene);
    let wood = kit::surface_tinted(&mut scene, "planks", [0.8, 0.72, 0.62]);
    let block = |scene: &mut RenderScene, min: Vec3, max: Vec3| {
        let mesh = scene.add_mesh(kit::block(min, max, 1.2));
        scene.add_instance(mesh, wood, Mat4::IDENTITY);
    };
    let (x0, x1, z0, z1, top) = (-2.4f32, 2.4f32, -6.0f32, -2.0f32, 2.6f32);
    for x in [x0, 0.0, x1] {
        for z in [z0, z1] {
            block(
                &mut scene,
                Vec3::new(x - 0.1, 0.0, z - 0.1),
                Vec3::new(x + 0.1, top, z + 0.1),
            );
        }
        block(
            &mut scene,
            Vec3::new(x - 0.08, top, z0 - 0.4),
            Vec3::new(x + 0.08, top + 0.18, z1 + 0.4),
        );
    }
    for k in 0..7 {
        let z = z0 - 0.2 + (z1 - z0 + 0.4) * k as f32 / 6.0;
        block(
            &mut scene,
            Vec3::new(x0 - 0.4, top + 0.18, z - 0.04),
            Vec3::new(x1 + 0.4, top + 0.26, z + 0.04),
        );
    }
    // The table and its benches.
    block(
        &mut scene,
        Vec3::new(-1.4, 0.72, -4.4),
        Vec3::new(1.4, 0.78, -3.6),
    );
    for x in [-1.25f32, 1.25] {
        block(
            &mut scene,
            Vec3::new(x - 0.06, 0.0, -4.3),
            Vec3::new(x + 0.06, 0.72, -3.7),
        );
    }
    for z in [-4.9f32, -3.1] {
        block(
            &mut scene,
            Vec3::new(-1.3, 0.42, z - 0.15),
            Vec3::new(1.3, 0.47, z + 0.15),
        );
        for x in [-1.15f32, 1.15] {
            block(
                &mut scene,
                Vec3::new(x - 0.05, 0.0, z - 0.12),
                Vec3::new(x + 0.05, 0.42, z + 0.12),
            );
        }
    }
    let lantern = kit::prop(&mut scene, "toro");
    lantern.place(&mut scene, Vec3::new(3.2, 0.0, -2.4), 0.4, 1.6);
    lantern.place(&mut scene, Vec3::new(-3.2, 0.0, -2.4), -0.4, 1.6);
    // The vines over the slats: two layers of leaf cards, alpha-tested.
    let leaves = scene.add_texture(vine_leaves());
    let leaf = scene.add_standard_material(og_render::StandardMaterial {
        color: Vec3::ONE,
        roughness: 0.7,
        side: 2,
        map: leaves.slot(),
        alphaTest: 0.5,
        ..Default::default()
    });
    for (y, shift) in [(top + 0.3, 0.0f32), (top + 0.42, 0.37)] {
        let mut card = kit::slab(
            Vec2::new(x0 - 0.6, z0 - 0.6),
            Vec2::new(x1 + 0.6, z1 + 0.6),
            y,
            2.2,
        );
        for v in &mut card.vertices {
            v.uv = [v.uv[0] + shift, v.uv[1] + shift * 0.6];
        }
        let mesh = scene.add_mesh(card);
        scene.add_instance(mesh, leaf, Mat4::IDENTITY);
    }
    physical_light_under(&mut scene, Vec3::new(0.3, 0.8, -0.5));
    let light = &mut scene.lighting.light;
    light.shadow_box = 8.0;
    light.shadow_focus = 12.0;
    light.shadow_focus_drop = 6.0;
    light.shadow_bias = 0.04;
    light.shadow_normal_bias = 0.04;
    scene.frame.camera = Camera {
        position: Vec3::new(2.6, 2.2, 3.0),
        target: Vec3::new(-0.8, 0.9, -4.6),
        far: 2500.0,
        exposure: Exposure::Manual { ev: -0.3 },
        ..Default::default()
    };
    scene
}

/// `hemisphere-light`: a stone loggia on a lawn under an overcast, lit by
/// no sun: the hemisphere light gives every surface the sky's cool colour
/// as it faces up and the sun-warmed ground's colour as it faces down, so a
/// pale ball under the columns reads cool on top and warm beneath, the
/// loggia's floor cool and its vault's underside warm.
pub fn hemisphere_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    crate::shadow_scenes::meadow(&mut scene, None);
    physical_light_under(&mut scene, Vec3::new(0.3, 0.6, -0.5));
    // An overcast: the deck dims the sky's light as every clouded scene
    // is lit (`Clouds::light_scene`; the deck's own light follows the
    // environment's intensity, so it is left at the look's). The sun
    // behind it is put out, so the hemisphere light carries the shot.
    let clouds = og_render::Clouds {
        coverage: 0.95,
        color: Vec3::splat(0.9),
        base: 900.0,
        layer: 0.0,
        ..og_render::Clouds::default()
    };
    scene.atmosphere.clouds = Some(clouds);
    clouds.light_scene(&mut scene);
    scene.lighting.light.intensity = 0.0;
    scene.lighting.light.cast_shadows = false;
    scene.lighting.hemisphere = ([0.55, 0.64, 0.82], [0.95, 0.58, 0.3], 2.0);
    let stone = kit::surface_tinted(&mut scene, "stone", [0.95, 0.9, 0.84]);
    let column = kit::prop(&mut scene, "column");
    for k in 0..6 {
        column.place(
            &mut scene,
            Vec3::new(-10.0 + 4.0 * k as f32, 0.3, -9.0),
            0.3 * k as f32,
            5.2,
        );
    }
    for (min, max) in [
        (Vec3::new(-12.0, 5.5, -16.0), Vec3::new(12.0, 6.3, -8.0)),
        (Vec3::new(-12.0, 0.0, -16.0), Vec3::new(12.0, 0.3, -8.0)),
        (Vec3::new(-12.0, 0.3, -16.0), Vec3::new(12.0, 5.5, -15.4)),
    ] {
        let mesh = scene.add_mesh(kit::block(min, max, 2.4));
        scene.add_instance(mesh, stone, Mat4::IDENTITY);
    }
    kit::prop(&mut scene, "stone_bench").place(&mut scene, Vec3::new(1.4, 0.3, -14.4), 0.0, 0.48);
    // Under the columns, a pale ball on a plinth: its top takes the sky's
    // cool, its underside the warm ground's, the gradient between them the
    // hemisphere light itself.
    let plinth = scene.add_mesh(kit::block(
        Vec3::new(-2.1, 0.3, -12.6),
        Vec3::new(-0.9, 0.95, -11.4),
        1.0,
    ));
    scene.add_instance(plinth, stone, Mat4::IDENTITY);
    let ball = scene.add_mesh(shapes::sphere(48, 32));
    let pale = scene.add_standard_material(og_render::StandardMaterial {
        color: Vec3::splat(0.6),
        roughness: 0.45,
        ..Default::default()
    });
    scene.add_instance(
        ball,
        pale,
        Mat4::from_scale_rotation_translation(
            Vec3::splat(1.1),
            Quat::IDENTITY,
            Vec3::new(-1.5, 1.5, -12.0),
        ),
    );
    scene.frame.camera = Camera {
        position: Vec3::new(-4.8, 1.2, -9.4),
        target: Vec3::new(-1.4, 1.4, -12.1),
        far: 2500.0,
        exposure: Exposure::Manual { ev: -0.6 },
        ..Default::default()
    };
    scene
}
