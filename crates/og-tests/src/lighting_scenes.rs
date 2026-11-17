//! The lighting and shadows scenes as shots of a game: the old town's high
//! street by morning, noon, dusk and night, its market, a crypt and the
//! ruined colonnade, each lit so the feature its scene is named for is the
//! subject of the shot. Where a test measures a scene's pixels it keeps its
//! own plain fixture (`scenes::cascades_200m`, `scenes::dim_scene`,
//! `scenes::point_spot_lights`, `p2_scenes::many_lights`,
//! `shadow_scenes::local_contact_scene`, `fork_scenes::shadow_box_scene`
//! and `hemisphere_scene`); the shots here are the registered scenes.

use crate::kit;
use crate::sky_scenes::physical_light_under;
use glam::{Mat4, Quat, Vec2, Vec3};
use og_render::{Camera, Exposure, LensFlare, Light, LightKind, RenderScene, shapes};

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
        scene.lighting.lights.push(Light {
            kind: LightKind::Point,
            position: l.glass - Vec3::new(l.side * 0.3, 0.25, 0.0),
            color: Vec3::new(1.0, 0.72, 0.45),
            intensity,
            range: 16.0,
            cast_shadows: i < shadowed,
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

/// `cascades-200m`: the high street on a summer morning, 200 m of it in one
/// view down to the watchtower: crates and barrels of a market at the
/// camera's feet throw small sharp shadows in the first cascade, the lamp
/// posts and the house fronts' eaves stripe the setts in the middle ones,
/// and the far houses still shade the street 190 m off in the last.
pub fn cascades_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    high_street(&mut scene, 200.0);
    wares(&mut scene, Vec3::new(-2.4, 0.0, -2.0), 1.4);
    wares(&mut scene, Vec3::new(2.6, 0.0, -14.0), -1.7);
    // The sun behind the camera and over the right-hand roofs: the right
    // row's shadow runs down the street's right half to its end, its edge
    // the eaves' saw-tooth, and the lamp posts and wares on the sunlit left
    // half throw theirs away from the camera.
    physical_light_under(&mut scene, Vec3::new(0.36, 0.52, 0.8));
    scene.lighting.light.max_shadow_distance = 220.0;
    scene.frame.camera = Camera {
        position: Vec3::new(-1.0, 2.6, 6.0),
        target: Vec3::new(0.4, 3.2, -60.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: -0.3 },
        ..Default::default()
    };
    scene
}

/// `exposure`: the high street at deep dusk, the sun gone below the roofs
/// and the lamps just lit: a view a fixed exposure leaves near black,
/// brought to middle grey by the metered one.
pub fn exposure_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    let lanterns = high_street(&mut scene, 120.0);
    wares(&mut scene, Vec3::new(-2.4, 0.0, -4.0), 1.4);
    light_lanterns(&mut scene, &lanterns[..8], 10.0, 0, 0.25);
    physical_light_under(&mut scene, Vec3::new(-0.7, -0.035, -0.71));
    scene.frame.camera = Camera {
        position: Vec3::new(1.2, 1.7, 7.0),
        target: Vec3::new(-0.6, 3.0, -40.0),
        far: 2500.0,
        exposure: Exposure::auto(),
        ..Default::default()
    };
    scene
}

/// `flare-noon`: in the street at noon, the camera on the setts looking
/// up past the eaves at a sun almost overhead: its ghosts and halo cross
/// the gap of sky between the roofs.
pub fn flare_noon_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    high_street(&mut scene, 80.0);
    wares(&mut scene, Vec3::new(-2.6, 0.0, -1.0), 1.4);
    let sun = Vec3::new(0.12, 1.0, -0.35).normalize();
    physical_light_under(&mut scene, sun);
    scene.look.post.lens_flare = Some(LensFlare::default());
    // Looking up the street at the sun, which stands in the upper right
    // of the gap of sky between the eaves.
    let eye = Vec3::new(0.4, 1.7, 4.0);
    scene.frame.camera = Camera {
        position: eye,
        target: eye + sun + Vec3::new(-0.05, -0.18, -0.12),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.4 },
        ..Default::default()
    };
    scene
}

/// `flare-edge`: a sun at `elevation` (the slope of its direction) low
/// down the high street, behind the top of a gate tower that closes it:
/// the tower's battlements cross the sun's disc (at 0.3, through its
/// middle), the camera turned a little right of it, so the flare comes from
/// the part of the disc in view.
pub fn flare_edge_scene(elevation: f32) -> RenderScene {
    let mut scene = RenderScene::new();
    high_street(&mut scene, 60.0);
    let eye = Vec3::new(1.4, 1.7, 2.0);
    let along = Vec3::new(-0.04, 0.0, -1.0).normalize();
    let sun = (along + Vec3::Y * elevation).normalize();
    physical_light_under(&mut scene, sun);
    // The tower on the sun's line from the eye, as far off as puts its
    // top on that line.
    let tower = kit::prop(&mut scene, "watchtower");
    // The line runs 15 cm under the battlements' top: a little more than
    // half of the disc (60 cm across there) hides behind them.
    let h = 22.0;
    let d = (h - 0.15 - eye.y) / elevation;
    let foot = eye + along * d;
    tower.place(&mut scene, Vec3::new(foot.x, 0.0, foot.z), 0.0, h);
    scene.look.post.lens_flare = Some(LensFlare::default());
    let right = Vec3::new(-along.z, 0.0, along.x);
    scene.frame.camera = Camera {
        position: eye,
        target: eye + along + right * 0.22 + Vec3::Y * (elevation - 0.06),
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
        scene.lighting.lights.push(Light {
            kind: LightKind::Spot {
                direction: (ahead - Vec3::Y * 0.12).normalize(),
                angle_degrees: 30.0,
                penumbra: 0.35,
            },
            position: p + ahead * 0.1,
            color: Vec3::new(1.0, 0.95, 0.85),
            intensity,
            range: 30.0,
            cast_shadows: false,
        });
    }
    lights
}

/// `point-spot-lights`: the high street at night, seven lights: the two
/// nearest lanterns shadowed (the crates and barrels of a stall between
/// them throw their shadows across the setts), two more lanterns down the
/// street, a car pulled in at the kerb with its headlights on (two spots)
/// and a spot over a doorway on the right.
pub fn point_spot_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    let lanterns = high_street(&mut scene, 80.0);
    night(&mut scene);
    // The nearest lantern on each side, then the next two.
    light_lanterns(&mut scene, &lanterns[..4], 24.0, 2, 3.0);
    wares(&mut scene, Vec3::new(-2.0, 0.0, -2.5), 1.5);
    car_with_headlights(
        &mut scene,
        Vec3::new(2.4, 0.0, -14.0),
        Vec2::new(-0.18, 1.0),
        120.0,
    );
    scene.lighting.lights.push(Light {
        kind: LightKind::Spot {
            direction: Vec3::new(-0.35, -1.0, 0.1).normalize(),
            angle_degrees: 40.0,
            penumbra: 0.5,
        },
        position: Vec3::new(5.9, 3.4, -24.0),
        color: Vec3::new(1.0, 0.82, 0.6),
        intensity: 60.0,
        range: 10.0,
        cast_shadows: false,
    });
    scene.frame.camera = Camera {
        position: Vec3::new(0.6, 2.2, 7.5),
        target: Vec3::new(-0.4, 1.4, -14.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.2 },
        ..Default::default()
    };
    scene
}

/// `flare-lights`: down the high street at night: lanterns at different
/// distances flaring through their own discs, one half hidden behind a
/// nearer lamp post, and a car coming up the street with its headlights
/// in the lens.
pub fn flare_lights_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    let lanterns = high_street(&mut scene, 80.0);
    night(&mut scene);
    light_lanterns(&mut scene, &lanterns[..8], 24.0, 0, 3.0);
    car_with_headlights(
        &mut scene,
        Vec3::new(-1.6, 0.0, -18.0),
        Vec2::new(0.06, 1.0),
        120.0,
    );
    // The eye lined up behind the second left lamp post with the fourth
    // left lantern: the post's shaft hides the left half of its glass.
    let left: Vec<Lantern> = lanterns.iter().filter(|l| l.side < 0.0).copied().collect();
    let (post, far) = (left[1].glass, left[3].glass);
    let eye = Vec3::new(-1.2, 1.7, 6.0);
    let mut eye_at = eye;
    // Slide the eye sideways so the post's shaft (at the post's x) falls on
    // the line from the eye to the far glass's left edge.
    let t = (post.z - eye.z) / (far.z - eye.z);
    eye_at.x = (post.x + 0.05 - t * (far.x - 0.12)) / (1.0 - t);
    scene.look.post.lens_flare = Some(LensFlare::default());
    scene.frame.camera = Camera {
        position: eye_at,
        target: Vec3::new(0.4, 2.2, -30.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: -0.4 },
        ..Default::default()
    };
    scene
}

/// `lights-256`: a festival night in the old town's square, lit by 256
/// lights (Forward+): 224 paper lanterns on fourteen strings sagging from
/// pole to pole across the square, each a point light in its own colour,
/// and 32 spots at the house fronts' feet washing them upward; a market
/// stall and its wares under the strings.
pub fn festival_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    market_square(&mut scene);
    night(&mut scene);
    crate::sky_scenes::canopy(&mut scene, false);
    wares(&mut scene, Vec3::new(-8.0, 0.0, 2.0), 0.6);
    wares(&mut scene, Vec3::new(9.0, 0.0, -1.0), -0.9);
    let wood = kit::surface(&mut scene, "planks");
    let paper = [
        Vec3::new(1.0, 0.25, 0.12),
        Vec3::new(1.0, 0.55, 0.15),
        Vec3::new(1.0, 0.8, 0.35),
        Vec3::new(1.0, 0.35, 0.45),
        Vec3::new(0.95, 0.9, 0.7),
        Vec3::new(0.35, 0.8, 0.45),
        Vec3::new(0.4, 0.55, 1.0),
    ];
    let globe = scene.add_mesh(shapes::sphere(10, 8));
    let glows: Vec<_> = paper
        .iter()
        .map(|c| {
            scene.add_standard_material(og_render::StandardMaterial {
                color: Vec3::ZERO,
                emissive: *c,
                emissiveIntensity: 2.5,
                ..Default::default()
            })
        })
        .collect();
    let mut lights = Vec::with_capacity(256);
    for row in 0..14 {
        let z = 8.0 - 2.6 * row as f32;
        let (x0, x1) = (-22.0f32, 22.0f32);
        for x in [x0, x1] {
            let pole = scene.add_mesh(kit::block(
                Vec3::new(x - 0.09, 0.0, z - 0.09),
                Vec3::new(x + 0.09, 5.0, z + 0.09),
                1.5,
            ));
            scene.add_instance(pole, wood, Mat4::IDENTITY);
        }
        for k in 0..16 {
            let t = (k as f32 + 0.5) / 16.0;
            let sag = 1.6 * 4.0 * t * (1.0 - t);
            let p = Vec3::new(x0 + (x1 - x0) * t, 4.8 - sag, z);
            let pick = (crate::fx_scenes::hash2(row, k, 0xfe5) * paper.len() as f32) as usize;
            let c = paper[pick.min(paper.len() - 1)];
            scene.add_instance(
                globe,
                glows[pick.min(paper.len() - 1)],
                Mat4::from_scale_rotation_translation(Vec3::new(0.3, 0.36, 0.3), Quat::IDENTITY, p),
            );
            lights.push(Light {
                kind: LightKind::Point,
                position: p - Vec3::Y * 0.25,
                color: c,
                intensity: 9.0,
                range: 8.0,
                cast_shadows: false,
            });
        }
    }
    // Spots at the back row's and the right row's feet, washing them up.
    for k in 0..32 {
        let (position, direction) = if k < 20 {
            (
                Vec3::new(-31.0 + 3.1 * k as f32, 0.3, -24.0),
                Vec3::new(0.0, 1.0, -0.35),
            )
        } else {
            let j = (k - 20) as f32;
            (
                Vec3::new(27.0, 0.3, -17.0 + 3.6 * j),
                Vec3::new(0.35, 1.0, 0.0),
            )
        };
        lights.push(Light {
            kind: LightKind::Spot {
                direction: direction.normalize(),
                angle_degrees: 34.0,
                penumbra: 0.5,
            },
            position,
            color: Vec3::new(1.0, 0.78, 0.55),
            intensity: 40.0,
            range: 14.0,
            cast_shadows: false,
        });
    }
    scene.lighting.lights = lights;
    scene.frame.camera = Camera {
        position: Vec3::new(-3.0, 6.5, 22.0),
        target: Vec3::new(1.0, 2.5, -10.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: -1.0 },
        ..Default::default()
    };
    scene
}

/// `shadows-local-contact`: a wine cellar lit by one shadowed spot, a
/// lantern hung from a beam up and to the left, aimed at a crate standing
/// on the plank floor: the crate's and the barrels' shadows start at their
/// bases with no lit line under them, their edges soft and straight (the
/// tent PCF). Rubble walls, a beamed plank ceiling, a ladder to the
/// trapdoor.
pub fn cellar_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    let planks = kit::surface_tinted(&mut scene, "planks", [0.85, 0.8, 0.75]);
    let stone = kit::surface_tinted(&mut scene, "stone", [0.95, 0.9, 0.84]);
    let floor = scene.add_mesh(kit::slab(
        Vec2::new(-5.0, -3.0),
        Vec2::new(5.0, 6.0),
        0.0,
        2.2,
    ));
    scene.add_instance(floor, planks, Mat4::IDENTITY);
    for (min, max) in [
        (Vec3::new(-5.6, 0.0, -3.6), Vec3::new(5.6, 3.2, -3.0)),
        (Vec3::new(-5.6, 0.0, -3.0), Vec3::new(-5.0, 3.2, 6.0)),
        (Vec3::new(5.0, 0.0, -3.0), Vec3::new(5.6, 3.2, 6.0)),
    ] {
        let wall = scene.add_mesh(kit::block(min, max, 2.4));
        scene.add_instance(wall, stone, Mat4::IDENTITY);
    }
    let ceiling = scene.add_mesh(kit::block(
        Vec3::new(-5.6, 3.2, -3.6),
        Vec3::new(5.6, 3.35, 6.0),
        2.2,
    ));
    scene.add_instance(ceiling, planks, Mat4::IDENTITY);
    for k in 0..5 {
        let x = -4.0 + 2.0 * k as f32;
        let beam = scene.add_mesh(kit::block(
            Vec3::new(x - 0.12, 2.95, -3.0),
            Vec3::new(x + 0.12, 3.2, 6.0),
            1.2,
        ));
        scene.add_instance(beam, planks, Mat4::IDENTITY);
    }
    let barrel = kit::prop(&mut scene, "winebarrel");
    for (k, x) in [-4.3f32, -3.4, -2.5, 2.6, 3.5].into_iter().enumerate() {
        barrel.place(&mut scene, Vec3::new(x, 0.0, -2.5), k as f32 * 1.3, 1.05);
    }
    barrel.place(&mut scene, Vec3::new(-3.85, 1.0, -2.5), 0.4, 1.05);
    let crate_ = kit::prop(&mut scene, "crate");
    crate_.place(&mut scene, Vec3::new(0.0, 0.0, 0.0), 0.15, 0.62);
    crate_.place(&mut scene, Vec3::new(1.4, 0.0, -2.3), -0.2, 0.62);
    crate_.place(&mut scene, Vec3::new(1.45, 0.62, -2.35), 0.1, 0.62);
    barrel.place(&mut scene, Vec3::new(-1.0, 0.0, 0.9), 2.0, 1.0);
    let ladder = kit::prop(&mut scene, "ladder");
    let lean = Mat4::from_translation(Vec3::new(4.3, 0.0, 1.5))
        * Mat4::from_rotation_y(std::f32::consts::FRAC_PI_2)
        * Mat4::from_rotation_x(-0.22);
    ladder.place_by(&mut scene, lean * ladder.transform(Vec3::ZERO, 0.0, 3.4));
    // The lantern on its hook: its glass lit, casting nothing itself.
    let position = Vec3::new(-1.6, 2.6, 0.6);
    let glass = scene.add_standard_material(og_render::StandardMaterial {
        color: Vec3::ZERO,
        emissive: Vec3::new(1.0, 0.75, 0.45),
        emissiveIntensity: 0.5,
        ..Default::default()
    });
    let lantern = scene.add_mesh(shapes::cuboid(Vec3::new(0.16, 0.22, 0.16)));
    scene.add_instance(
        lantern,
        glass,
        Mat4::from_translation(position + Vec3::Y * 0.1),
    );
    if let Some(last) = scene.objects.instances.last_mut() {
        last.cast_shadows = false;
    }
    scene.lighting.lights.push(Light {
        kind: LightKind::Spot {
            direction: (Vec3::new(0.4, 0.0, 0.0) - position).normalize(),
            angle_degrees: 60.0,
            penumbra: 0.35,
        },
        position,
        color: Vec3::new(1.0, 0.8, 0.55),
        intensity: 40.0,
        range: 12.0,
        cast_shadows: true,
    });
    scene.lighting.light.intensity = 0.0;
    scene.lighting.light.cast_shadows = false;
    scene.lighting.environment = og_render::Environment {
        intensity: 0.02,
        background: og_render::Background::Color(Vec3::new(0.01, 0.008, 0.006)),
        ..Default::default()
    };
    scene.frame.camera = Camera {
        position: Vec3::new(1.6, 1.9, 4.6),
        target: Vec3::new(-0.2, 0.5, -0.8),
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
    for k in 0..250 {
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

/// `shadow-box`: a vine pergola in a corner of the old town's square on a
/// summer afternoon, drawn under one 8 m shadow box (the cheap tier's sun
/// shadows instead of cascades) focused 12 m ahead of the camera, on the
/// pergola: its posts, the table and benches under it, and the dappled
/// shade of its alpha-tested leaves are sharp, while the square beyond
/// the box goes unshadowed.
pub fn shadow_box_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    market_square(&mut scene);
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
    let barrel = kit::prop(&mut scene, "winebarrel");
    barrel.place(&mut scene, Vec3::new(3.1, 0.0, -2.6), 0.4, 1.0);
    let crate_ = kit::prop(&mut scene, "crate");
    crate_.place(&mut scene, Vec3::new(-3.1, 0.0, -2.8), 0.6, 0.6);
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

/// `hemisphere-light`: a stone loggia on the old town's square under a
/// grey sky, lit by no sun: the hemisphere light gives every surface the
/// sky's cool colour as it faces up and the sun-warmed setts' colour as it
/// faces down, so the loggia's floor and the column tops read cool and its
/// vault's underside and the columns' undersides warm.
pub fn hemisphere_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    market_square(&mut scene);
    physical_light_under(&mut scene, Vec3::new(0.3, 0.6, -0.5));
    scene.lighting.light.intensity = 0.0;
    scene.lighting.light.cast_shadows = false;
    scene.lighting.environment.intensity = 0.25;
    scene.atmosphere.clouds = Some(og_render::Clouds {
        coverage: 0.9,
        color: Vec3::splat(0.7),
        base: 900.0,
        layer: 0.0,
        ..og_render::Clouds::default()
    });
    scene.lighting.hemisphere = ([0.55, 0.64, 0.8], [0.9, 0.6, 0.36], 2.0);
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
    wares(&mut scene, Vec3::new(-5.0, 0.3, -12.5), 0.0);
    wares(&mut scene, Vec3::new(4.0, 0.3, -13.0), 0.2);
    scene.frame.camera = Camera {
        position: Vec3::new(-9.0, 1.9, -10.2),
        target: Vec3::new(4.0, 3.4, -11.6),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.8 },
        ..Default::default()
    };
    scene
}
